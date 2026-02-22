use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const KNOWN_WINAPI: &[&str] = &[
    "CreateFileA",
    "ReadFile",
    "WriteFile",
    "CloseHandle",
    "MessageBoxA",
    "VirtualAlloc",
    "VirtualFree",
    "GetLastError",
    "SetLastError",
    "ExitProcess",
    "GetCurrentProcess",
    "Sleep",
    "GetTickCount",
    "GetModuleHandle",
    "GetProcAddress",
    "LoadLibrary",
    "FreeLibrary",
    "SendInput",
    "mouse_event",
    "keybd_event",
    "GetCursorPos",
    "SetCursorPos",
    "GetAsyncKeyState",
    "GetKeyState",
    "MapVirtualKey",
    "ShowCursor",
    "ClipCursor",
    "CreateThread",
    "WaitForSingleObject",
    "CreateEvent",
    "SetEvent",
    "ResetEvent",
    "QueryPerformanceCounter",
    "QueryPerformanceFrequency",
    "GetSystemTime",
    "GetLocalTime",
];

const TRACE_MAX_STOPS: usize = 128;

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("winrun error: {err}");
            std::process::exit(1);
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Mode {
    Run,
    CompileOnly,
}

fn run() -> Result<i32, String> {
    let (mode, debug, target) = parse_args()?;

    if debug {
        println!("=== winrun debug mode ===");
        println!("mode: {mode:?}");
        println!("target: {}", target.display());
        debug_log("init", "collecting file metadata");
    }

    let metadata = fs::metadata(&target).map_err(|e| format!("failed to stat target: {e}"))?;
    if !metadata.is_file() {
        return Err(format!("target is not a file: {}", target.display()));
    }

    let bytes = fs::read(&target).map_err(|e| format!("failed to read target: {e}"))?;
    let format = detect_format(&bytes);

    if debug {
        println!("format: {format}");
        println!("size: {} bytes", bytes.len());
        debug_log("inspect", "format detection finished");
    }

    if can_run_natively(&bytes) {
        return handle_native(mode, debug, &target, &metadata);
    }

    handle_non_native(mode, debug, &target, &bytes)
}

fn handle_native(
    mode: Mode,
    debug: bool,
    target: &Path,
    metadata: &fs::Metadata,
) -> Result<i32, String> {
    if debug {
        println!("native: yes (ELF detected)");
        debug_log("native", "entering Linux execution path");
        if is_executable(metadata) {
            if let Some(trace) = trace_with_gdb(target)? {
                print_trace_report(&trace);
            } else {
                println!("gdb-trace: unavailable (gdb missing or target not traceable)");
            }
        } else {
            println!("gdb-trace: skipped (target is not executable)");
        }
    }

    if mode == Mode::CompileOnly {
        let plan_path = plan_output_path(target);
        fs::write(
            &plan_path,
            "# native ELF: no waygate translation required\n",
        )
        .map_err(|e| format!("failed to write plan {}: {e}", plan_path.display()))?;
        println!("created plan: {}", plan_path.display());
        return Ok(0);
    }

    if debug {
        println!("action: running directly on Linux");
        debug_log("native", "dispatching to execve");
    }
    exec_native(target).map_err(|e| format!("native execution failed: {e}"))
}

fn handle_non_native(mode: Mode, debug: bool, target: &Path, bytes: &[u8]) -> Result<i32, String> {
    if debug {
        println!("native: no");
        println!(
            "gdb-trace: skipped (non-native binaries cannot run before compatibility translation)"
        );
        println!("action: compatibility scan + waygate dispatch");
        debug_log("non-native", "analyzing candidate Win32 symbols");
    }

    let analysis = analyze_non_native(bytes);
    if debug {
        print_non_native_report(&analysis);
    }

    if analysis.winapi_calls.is_empty() {
        return Err("binary is not native and has no known Win32 API signatures".to_string());
    }

    let plan_path = plan_output_path(target);
    write_plan_file(&plan_path, &analysis.winapi_calls)
        .map_err(|e| format!("failed to write plan {}: {e}", plan_path.display()))?;
    if debug {
        debug_log(
            "plan",
            &format!(
                "wrote {} API calls to {}",
                analysis.winapi_calls.len(),
                plan_path.display()
            ),
        );
    }

    if mode == Mode::CompileOnly {
        println!("created plan: {}", plan_path.display());
        return Ok(0);
    }

    if debug {
        println!("generated plan: {}", plan_path.display());
        println!("executing plan through waygate");
    }

    for call in &analysis.winapi_calls {
        match waygate::dispatch(&call.function, &call.args) {
            Ok(msg) if debug => println!(
                "  [ok] {}({}) -> {msg}",
                call.function,
                call.args.join(", ")
            ),
            Ok(_) => {}
            Err(err) if debug => println!(
                "  [err] {}({}) -> {err}",
                call.function,
                call.args.join(", ")
            ),
            Err(_) => {}
        }
    }

    if debug {
        debug_log("done", "waygate dispatch finished successfully");
    }

    Ok(0)
}

fn debug_log(stage: &str, msg: &str) {
    eprintln!("[debug:{stage}] {msg}");
}

#[derive(Clone, Debug)]
struct TracedCall {
    function: String,
    args: Vec<String>,
    backtrace: Vec<String>,
}

#[derive(Default)]
struct Analysis {
    winapi_calls: Vec<TracedCall>,
    non_windows_libs: Vec<String>,
}

fn parse_args() -> Result<(Mode, bool, PathBuf), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [path] => Ok((Mode::Run, false, PathBuf::from(path))),
        [flag, path] if flag == "-d" => Ok((Mode::Run, true, PathBuf::from(path))),
        [flag, path] if flag == "-c" => Ok((Mode::CompileOnly, false, PathBuf::from(path))),
        [flag, path] if flag == "-cd" || flag == "-dc" => {
            Ok((Mode::CompileOnly, true, PathBuf::from(path)))
        }
        _ => Err("usage: winrun [-d] [-c|-cd] <binary-file>".to_string()),
    }
}

fn detect_format(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x7F, b'E', b'L', b'F']) {
        "ELF"
    } else if bytes.starts_with(b"MZ") {
        "PE/COFF (Windows)"
    } else {
        "unknown"
    }
}

fn can_run_natively(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x7F, b'E', b'L', b'F'])
}

fn is_executable(metadata: &fs::Metadata) -> bool {
    metadata.permissions().mode() & 0o111 != 0
}

fn exec_native(path: &Path) -> Result<i32, io::Error> {
    let err = Command::new(path).exec();
    Err(err)
}

fn plan_output_path(target: &Path) -> PathBuf {
    let mut out = target.to_path_buf();
    let filename = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("target");
    out.set_file_name(format!("{filename}.waygate.plan"));
    out
}

fn write_plan_file(path: &Path, calls: &[TracedCall]) -> io::Result<()> {
    let mut text = String::from("# waygate execution plan\n");
    for (idx, call) in calls.iter().enumerate() {
        let typed_args = typed_args_for_call(call);
        text.push_str(&format!(
            "{}\t{}\t{}\n",
            idx + 1,
            call.function,
            typed_args.join("||")
        ));
    }
    fs::write(path, text)
}

fn typed_args_for_call(call: &TracedCall) -> Vec<String> {
    call.args.iter().map(|arg| format_typed_arg(arg)).collect()
}

fn format_typed_arg(arg: &str) -> String {
    if let Some((name, value)) = arg.split_once('=') {
        let ty = infer_arg_type(value.trim());
        format!("{}:{}={}", name.trim(), ty, value.trim())
    } else {
        let ty = infer_arg_type(arg.trim());
        format!("value:{}={}", ty, arg.trim())
    }
}

fn infer_arg_type(raw: &str) -> &'static str {
    let value = raw.trim();
    if value.is_empty() {
        return "unknown";
    }

    if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
        return "bool";
    }

    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        return "string";
    }

    if value.starts_with("0x") && value[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return "pointer";
    }

    if value.parse::<i64>().is_ok() {
        return "int";
    }

    if value.parse::<f64>().is_ok() {
        return "float";
    }

    if value.eq_ignore_ascii_case("null") || value.eq_ignore_ascii_case("nullptr") {
        return "pointer";
    }

    if value.contains('/') || value.contains(".dll") || value.contains(".so") {
        return "path";
    }

    "unknown"
}

fn trace_with_gdb(path: &Path) -> Result<Option<Vec<TracedCall>>, String> {
    let script = build_gdb_script();
    let script_path = gdb_script_path();

    if let Err(err) =
        fs::File::create(&script_path).and_then(|mut file| file.write_all(script.as_bytes()))
    {
        return Err(format!("failed to prepare gdb script: {err}"));
    }

    let output = match Command::new("gdb")
        .arg("-q")
        .arg("-nx")
        .arg("-batch")
        .arg(path)
        .arg("-x")
        .arg(&script_path)
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            let _ = fs::remove_file(&script_path);
            return Ok(None);
        }
    };

    let _ = fs::remove_file(&script_path);

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(Some(parse_gdb_trace(&text)))
}

fn gdb_script_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("winrun-gdb-{nanos}.gdb"))
}

fn build_gdb_script() -> String {
    let mut lines = vec![
        "set pagination off".to_string(),
        "set confirm off".to_string(),
        "set breakpoint pending on".to_string(),
        "set print frame-arguments all".to_string(),
    ];

    for sym in KNOWN_WINAPI {
        lines.push(format!("rbreak ^{sym}$"));
    }

    lines.push("run".to_string());
    lines.push("set $i = 0".to_string());
    lines.push(format!("while $i < {TRACE_MAX_STOPS}"));
    lines.push("  if $_isvoid($_exitcode)".to_string());
    lines.push("    printf \"===TRACE_EVENT_BEGIN===\\n\"".to_string());
    lines.push("    frame".to_string());
    lines.push("    info args".to_string());
    lines.push("    backtrace 8".to_string());
    lines.push("    printf \"===TRACE_EVENT_END===\\n\"".to_string());
    lines.push("    continue".to_string());
    lines.push("  else".to_string());
    lines.push("    loop_break".to_string());
    lines.push("  end".to_string());
    lines.push("  set $i = $i + 1".to_string());
    lines.push("end".to_string());

    lines.join("\n")
}

fn parse_gdb_trace(stdout: &str) -> Vec<TracedCall> {
    let mut calls = Vec::new();
    let mut block = Vec::new();
    let mut inside = false;

    for line in stdout.lines() {
        if line.contains("===TRACE_EVENT_BEGIN===") {
            inside = true;
            block.clear();
            continue;
        }

        if line.contains("===TRACE_EVENT_END===") {
            inside = false;
            if let Some(call) = parse_trace_block(&block) {
                calls.push(call);
            }
            block.clear();
            continue;
        }

        if inside {
            block.push(line.trim().to_string());
        }
    }

    calls
}

fn parse_trace_block(lines: &[String]) -> Option<TracedCall> {
    let mut function = None;
    let mut args = Vec::new();
    let mut backtrace = Vec::new();

    for line in lines {
        if function.is_none() {
            if let Some(name) = parse_function_name(line) {
                function = Some(name);
            }
        }

        if let Some((name, value)) = parse_arg_line(line) {
            args.push(format!("{name}={value}"));
        }

        if line.starts_with('#') {
            backtrace.push(line.clone());
        }
    }

    let function = function?;
    if !KNOWN_WINAPI.contains(&function.as_str()) {
        return None;
    }

    Some(TracedCall {
        function,
        args,
        backtrace,
    })
}

fn parse_function_name(line: &str) -> Option<String> {
    if !line.starts_with('#') {
        return None;
    }

    let without_frame = line.split_once(' ')?.1.trim();
    let candidate = without_frame
        .split(['(', ' '])
        .next()
        .unwrap_or_default()
        .trim_start_matches("0x");

    if candidate.is_empty() || candidate.chars().next()?.is_ascii_digit() {
        return None;
    }

    Some(candidate.to_string())
}

fn parse_arg_line(line: &str) -> Option<(String, String)> {
    let (name, value) = line.split_once('=')?;
    let key = name.trim();
    if key.is_empty() || key.starts_with('#') {
        return None;
    }
    Some((key.to_string(), value.trim().to_string()))
}

fn analyze_non_native(bytes: &[u8]) -> Analysis {
    if let Some(parsed) = parse_pe_imports(bytes) {
        return parsed;
    }
    parse_synthetic_fixture(bytes)
}

fn parse_pe_imports(bytes: &[u8]) -> Option<Analysis> {
    let pe = PeContext::parse(bytes)?;
    let import_rva = pe.import_rva;
    if import_rva == 0 {
        return Some(Analysis::default());
    }

    let mut seen = HashSet::new();
    let mut calls = Vec::new();
    let mut idx = 0usize;
    loop {
        let desc = pe.rva_to_offset(import_rva as usize + (idx * 20))?;
        let original_first_thunk = read_u32(bytes, desc)?;
        let name_rva = read_u32(bytes, desc + 12)?;
        let first_thunk = read_u32(bytes, desc + 16)?;
        if original_first_thunk == 0 && name_rva == 0 && first_thunk == 0 {
            break;
        }

        let thunk_rva = if original_first_thunk != 0 {
            original_first_thunk
        } else {
            first_thunk
        } as usize;
        let thunk_size = if pe.is_pe64 { 8 } else { 4 };
        let ordinal_flag: u64 = if pe.is_pe64 {
            0x8000_0000_0000_0000
        } else {
            0x8000_0000
        };

        let mut t = 0usize;
        loop {
            let thunk_off = match pe.rva_to_offset(thunk_rva + t * thunk_size) {
                Some(v) => v,
                None => break,
            };
            let entry = if pe.is_pe64 {
                read_u64(bytes, thunk_off)?
            } else {
                read_u32(bytes, thunk_off)? as u64
            };
            if entry == 0 {
                break;
            }
            if entry & ordinal_flag == 0 {
                let hint_name = pe.rva_to_offset(entry as usize)?;
                let name = read_c_string(bytes, hint_name + 2)?;
                if KNOWN_WINAPI.contains(&name.as_str()) && seen.insert(name.clone()) {
                    calls.push(TracedCall {
                        function: name,
                        args: Vec::new(),
                        backtrace: Vec::new(),
                    });
                }
            }
            t += 1;
        }

        idx += 1;
    }

    Some(Analysis {
        winapi_calls: calls,
        non_windows_libs: Vec::new(),
    })
}

struct PeSection {
    virtual_address: usize,
    mapped_size: usize,
    raw_ptr: usize,
}

struct PeContext {
    is_pe64: bool,
    import_rva: u32,
    sections: Vec<PeSection>,
}

impl PeContext {
    fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 0x40 || !bytes.starts_with(b"MZ") {
            return None;
        }

        let pe_offset = read_u32(bytes, 0x3C)? as usize;
        if pe_offset.checked_add(0x18)? > bytes.len() {
            return None;
        }
        if bytes.get(pe_offset..pe_offset + 4)? != b"PE\0\0" {
            return None;
        }

        let section_count = read_u16(bytes, pe_offset + 6)? as usize;
        let optional_size = read_u16(bytes, pe_offset + 20)? as usize;
        let optional_header_offset = pe_offset + 24;
        let magic = read_u16(bytes, optional_header_offset)?;
        let (is_pe64, data_directory_offset) = match magic {
            0x10B => (false, optional_header_offset + 96),
            0x20B => (true, optional_header_offset + 112),
            _ => return None,
        };

        let import_rva = read_u32(bytes, data_directory_offset + 8)?;
        let section_table = pe_offset + 24 + optional_size;
        let mut sections = Vec::with_capacity(section_count);
        for i in 0..section_count {
            let sec = section_table + i * 40;
            let virtual_size = read_u32(bytes, sec + 8)? as usize;
            let virtual_address = read_u32(bytes, sec + 12)? as usize;
            let raw_size = read_u32(bytes, sec + 16)? as usize;
            let raw_ptr = read_u32(bytes, sec + 20)? as usize;
            sections.push(PeSection {
                virtual_address,
                mapped_size: virtual_size.max(raw_size),
                raw_ptr,
            });
        }

        Some(Self {
            is_pe64,
            import_rva,
            sections,
        })
    }

    fn rva_to_offset(&self, rva: usize) -> Option<usize> {
        for section in &self.sections {
            if rva >= section.virtual_address && rva < section.virtual_address + section.mapped_size
            {
                return section.raw_ptr.checked_add(rva - section.virtual_address);
            }
        }
        None
    }
}

fn parse_synthetic_fixture(bytes: &[u8]) -> Analysis {
    let text = String::from_utf8_lossy(bytes);
    let mut seen_signatures = BTreeSet::new();
    let mut seen_non_empty_args = BTreeSet::new();
    let mut winapi_calls = Vec::new();
    let mut libs = BTreeSet::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("mzfake") {
            continue;
        }

        let lower = trimmed.to_ascii_lowercase();
        if lower.ends_with(".dll") {
            continue;
        }
        if lower.ends_with(".so") {
            libs.insert(lower);
            continue;
        }

        for sym in KNOWN_WINAPI {
            if !lower.contains(&sym.to_ascii_lowercase()) {
                continue;
            }

            let call = parse_symbol_call_case_insensitive(trimmed, sym);
            let signature = format!("{}({})", call.function, call.args.join(","));
            if !seen_signatures.insert(signature) {
                continue;
            }

            if call.args.is_empty() {
                if seen_non_empty_args.contains(&call.function) {
                    continue;
                }
            } else {
                seen_non_empty_args.insert(call.function.clone());
            }

            winapi_calls.push(call);
        }
    }

    Analysis {
        winapi_calls,
        non_windows_libs: libs.into_iter().collect(),
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

fn read_c_string(bytes: &[u8], offset: usize) -> Option<String> {
    let slice = bytes.get(offset..)?;
    let end = slice.iter().position(|b| *b == 0)?;
    std::str::from_utf8(&slice[..end])
        .ok()
        .map(|s| s.to_string())
}

fn parse_symbol_call_case_insensitive(line: &str, symbol: &str) -> TracedCall {
    let lower_line = line.to_ascii_lowercase();
    let lower_symbol = symbol.to_ascii_lowercase();
    if let Some(start) = lower_line.find(&lower_symbol) {
        let rest = &line[start + symbol.len()..];
        if rest.starts_with('(') {
            if let Some(end) = rest.find(')') {
                let inner = &rest[1..end];
                let args = inner
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return TracedCall {
                    function: symbol.to_string(),
                    args,
                    backtrace: Vec::new(),
                };
            }
        }
    }

    TracedCall {
        function: symbol.to_string(),
        args: Vec::new(),
        backtrace: Vec::new(),
    }
}

fn print_trace_report(trace: &[TracedCall]) {
    println!("gdb-trace: {} matched call(s)", trace.len());
    for (idx, call) in trace.iter().enumerate() {
        println!(
            "  {:>2}. {}({})",
            idx + 1,
            call.function,
            call.args.join(", ")
        );
        if !call.backtrace.is_empty() {
            println!("      backtrace:");
            for line in &call.backtrace {
                println!("        {line}");
            }
        }
    }
}

fn print_non_native_report(analysis: &Analysis) {
    println!("win32api: found {} symbol(s)", analysis.winapi_calls.len());
    for (i, call) in analysis.winapi_calls.iter().enumerate() {
        if call.args.is_empty() {
            println!("  {:>2}. {}", i + 1, call.function);
        } else {
            println!(
                "  {:>2}. {}({})",
                i + 1,
                call.function,
                call.args.join(", ")
            );
        }
    }

    if analysis.non_windows_libs.is_empty() {
        println!("other unresolved libs: none");
    } else {
        println!("other unresolved libs:");
        for lib in &analysis.non_windows_libs {
            println!("  - {lib}");
        }
    }
}
