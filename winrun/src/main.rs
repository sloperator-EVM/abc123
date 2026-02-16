use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

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

    if !target.exists() {
        return Err(format!("input file not found: {}", target.display()));
    }

    let metadata = fs::metadata(&target).map_err(|e| format!("failed to read metadata: {e}"))?;
    if !metadata.is_file() {
        return Err("input path is not a regular file".to_string());
    }

    let bytes = fs::read(&target).map_err(|e| format!("failed to read file: {e}"))?;
    let format = detect_format(&bytes);

    if debug {
        println!("format: {format}");
        println!("size: {} bytes", bytes.len());
    }

    if can_run_natively(&bytes) {
        if debug {
            println!("native: yes (ELF detected)");
            println!("action: running directly on Linux");
        }
        return exec_native(&target).map_err(|e| format!("native execution failed: {e}"));
    }

    if debug {
        println!("native: no");
        println!("action: compatibility scan");
    }

    let analysis = analyze_binary(&target, &bytes, debug)?;

    if analysis.winapi_calls.is_empty() {
        if debug {
            println!("win32api: none found");
            println!("broken libs: {:?}", analysis.non_windows_libs);
        }
        return Err("binary is not native and has no supported Win32 API signatures".to_string());
    }

    if debug {
        println!("win32api: found {} symbol(s)", analysis.winapi_calls.len());
        for (i, sym) in analysis.winapi_calls.iter().enumerate() {
            println!("  {:>2}. {}", i + 1, sym);
        }
        if !analysis.non_windows_libs.is_empty() {
            println!("other unresolved libs:");
            for lib in &analysis.non_windows_libs {
                println!("  - {lib}");
            }
        }
        println!("dispatch: calling waygate stubs in observed order");
    }

    for sym in &analysis.winapi_calls {
        match waygate::dispatch(sym, &[]) {
            Ok(msg) => {
                if debug {
                    println!("  [ok] {sym} -> {msg}");
                }
            }
            Err(err) => {
                if debug {
                    println!("  [err] {sym} -> {err}");
                }
            }
        }
    }

    Ok(0)
}

fn parse_args() -> Result<(bool, PathBuf), String> {
    let mut args = env::args().skip(1);
    match (args.next(), args.next(), args.next()) {
        (Some(flag), Some(path), None) if flag == "-d" => Ok((true, PathBuf::from(path))),
        (Some(path), None, None) => Ok((false, PathBuf::from(path))),
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

#[derive(Default)]
struct Analysis {
    winapi_calls: Vec<String>,
    non_windows_libs: Vec<String>,
}

fn analyze_binary(path: &Path, bytes: &[u8], debug: bool) -> Result<Analysis, String> {
    let mut out = Analysis::default();

    let objdump = Command::new("objdump").arg("-p").arg(path).output();
    let mut text_blob = String::new();

    if let Ok(output) = objdump {
        if output.status.success() {
            text_blob.push_str(&String::from_utf8_lossy(&output.stdout));
        } else if debug {
            println!("note: objdump -p returned non-zero, using string scan fallback");
        }
    } else if debug {
        println!("note: objdump not available, using string scan fallback");
    }

    text_blob.push('\n');
    text_blob.push_str(&extract_ascii_strings(bytes));

    let mut ordered_symbols = Vec::new();
    for line in text_blob.lines() {
        for sym in KNOWN_WINAPI {
            if line.contains(sym) {
                ordered_symbols.push(sym.to_string());
            }
        }
    }

    let mut seen = BTreeSet::new();
    out.winapi_calls = ordered_symbols
        .into_iter()
        .filter(|s| seen.insert(s.clone()))
        .collect();

    let mut libs = BTreeSet::new();
    for tok in text_blob.split_whitespace() {
        let lower = tok
            .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.')
            .to_lowercase();
        if lower.ends_with(".dll") || lower.ends_with(".so") {
            let windows = lower.ends_with(".dll");
            if !windows {
                libs.insert(lower);
            }
        }
    }
    out.non_windows_libs = libs.into_iter().collect();

    Ok(out)
}

fn extract_ascii_strings(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut current = Vec::new();

    for &b in bytes {
        if b.is_ascii_graphic() || b == b' ' {
            current.push(b);
        } else {
            if current.len() >= 4 {
                if let Ok(s) = String::from_utf8(current.clone()) {
                    result.push_str(&s);
                    result.push('\n');
                }
            }
            current.clear();
        }
    }

    if current.len() >= 4 {
        if let Ok(s) = String::from_utf8(current) {
            result.push_str(&s);
            result.push('\n');
        }
    }

    result
}
