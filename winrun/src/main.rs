use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const KNOWN_WINAPI: &[&str] = &[
    "CreateFileA",
    "ReadFile",
    "WriteFile",
    "CloseHandle",
    "GetLastError",
    "MessageBoxA",
    "VirtualAlloc",
    "VirtualFree",
    "LoadLibraryA",
    "GetProcAddress",
    "ExitProcess",
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

fn run() -> Result<i32, String> {
    let (debug, target) = parse_args()?;

    if debug {
        println!("=== winrun debug mode ===");
        println!("target: {}", target.display());
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
    }

    if can_run_natively(&bytes) {
        if debug {
            println!("native: yes (ELF detected)");
            if let Some(trace) = trace_with_gdb(&target)? {
                print_trace_report(&trace);
            } else {
                println!("gdb-trace: unavailable (gdb missing or target not traceable)");
            }
            println!("action: running directly on Linux");
        }
        return exec_native(&target).map_err(|e| format!("native execution failed: {e}"));
    }

    if debug {
        println!("native: no");
        println!("action: compatibility scan + waygate dispatch");
    }

    let analysis = analyze_non_native(&bytes);

    if debug {
        print_non_native_report(&analysis);
    }

    if analysis.winapi_calls.is_empty() {
        return Err("binary is not native and has no known Win32 API signatures".to_string());
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

    Ok(0)
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

fn parse_args() -> Result<(bool, PathBuf), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [path] => Ok((false, PathBuf::from(path))),
        [flag, path] if flag == "-d" => Ok((true, PathBuf::from(path))),
        _ => Err("usage: winrun [-d] <binary-file>".to_string()),
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

fn exec_native(path: &Path) -> Result<i32, io::Error> {
    let err = Command::new(path).exec();
    Err(err)
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
    let strings = extract_ascii_strings(bytes);
    let mut ordered = Vec::new();

    for line in strings.lines() {
        for sym in KNOWN_WINAPI {
            if line.contains(sym) {
                ordered.push((*sym).to_string());
            }
        }
    }

    let mut seen = BTreeSet::new();
    let winapi_calls = ordered
        .into_iter()
        .filter(|s| seen.insert(s.clone()))
        .map(|function| TracedCall {
            function,
            args: Vec::new(),
            backtrace: Vec::new(),
        })
        .collect();

    let mut libs = BTreeSet::new();
    for tok in strings.split_whitespace() {
        let lower = tok
            .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.')
            .to_ascii_lowercase();
        if lower.ends_with(".so") {
            libs.insert(lower);
        }
    }

    Analysis {
        winapi_calls,
        non_windows_libs: libs.into_iter().collect(),
    }
}

fn extract_ascii_strings(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut start = None;

    for (i, b) in bytes.iter().enumerate() {
        if b.is_ascii_graphic() || *b == b' ' {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take() {
            if i.saturating_sub(s) >= 4 {
                let slice = &bytes[s..i];
                result.push_str(&String::from_utf8_lossy(slice));
                result.push('\n');
            }
        }
    }

    if let Some(s) = start {
        if bytes.len().saturating_sub(s) >= 4 {
            let slice = &bytes[s..];
            result.push_str(&String::from_utf8_lossy(slice));
            result.push('\n');
        }
    }

    result
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
        println!("  {:>2}. {}", i + 1, call.function);
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
