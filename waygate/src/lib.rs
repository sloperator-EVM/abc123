pub type WaygateResult = Result<String, String>;

pub fn dispatch(symbol: &str, args: &[String]) -> WaygateResult {
    match symbol {
        "CreateFileA" => stub("CreateFileA", args),
        "ReadFile" => stub("ReadFile", args),
        "WriteFile" => stub("WriteFile", args),
        "CloseHandle" => stub("CloseHandle", args),
        "MessageBoxA" => stub("MessageBoxA", args),
        "VirtualAlloc" => stub("VirtualAlloc", args),
        "VirtualFree" => stub("VirtualFree", args),
        "GetLastError" => stub("GetLastError", args),
        "SetLastError" => stub("SetLastError", args),
        "ExitProcess" => stub("ExitProcess", args),
        "GetCurrentProcess" => stub("GetCurrentProcess", args),
        "Sleep" => stub("Sleep", args),
        "GetTickCount" => stub("GetTickCount", args),
        "GetModuleHandle" => stub("GetModuleHandle", args),
        "GetProcAddress" => stub("GetProcAddress", args),
        "LoadLibrary" => stub("LoadLibrary", args),
        "FreeLibrary" => stub("FreeLibrary", args),
        "SendInput" => stub("SendInput", args),
        "mouse_event" => stub("mouse_event", args),
        "keybd_event" => stub("keybd_event", args),
        "GetCursorPos" => stub("GetCursorPos", args),
        "SetCursorPos" => stub("SetCursorPos", args),
        "GetAsyncKeyState" => stub("GetAsyncKeyState", args),
        "GetKeyState" => stub("GetKeyState", args),
        "MapVirtualKey" => stub("MapVirtualKey", args),
        "ShowCursor" => stub("ShowCursor", args),
        "ClipCursor" => stub("ClipCursor", args),
        "CreateThread" => stub("CreateThread", args),
        "WaitForSingleObject" => stub("WaitForSingleObject", args),
        "CreateEvent" => stub("CreateEvent", args),
        "SetEvent" => stub("SetEvent", args),
        "ResetEvent" => stub("ResetEvent", args),
        "QueryPerformanceCounter" => stub("QueryPerformanceCounter", args),
        "QueryPerformanceFrequency" => stub("QueryPerformanceFrequency", args),
        "GetSystemTime" => stub("GetSystemTime", args),
        "GetLocalTime" => stub("GetLocalTime", args),
        _ => Err(format!("waygate: symbol '{symbol}' is not implemented")),
    }
}

fn stub(name: &str, args: &[String]) -> WaygateResult {
    if args.is_empty() {
        Ok(format!("{name} stub called"))
    } else {
        Ok(format!("{name} stub called with args: {}", args.join(", ")))
    }
}
