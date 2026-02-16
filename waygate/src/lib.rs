use std::collections::HashMap;

pub type WaygateResult = Result<String, String>;

pub fn dispatch(symbol: &str, args: &[String]) -> WaygateResult {
    let registry = registry();
    if let Some(handler) = registry.get(symbol) {
        handler(args)
    } else {
        Err(format!("waygate: symbol '{}' is not implemented", symbol))
    }
}

fn registry() -> HashMap<&'static str, fn(&[String]) -> WaygateResult> {
    HashMap::from([
        (
            "CreateFileA",
            create_file_a as fn(&[String]) -> WaygateResult,
        ),
        ("ReadFile", read_file as fn(&[String]) -> WaygateResult),
        ("WriteFile", write_file as fn(&[String]) -> WaygateResult),
        (
            "CloseHandle",
            close_handle as fn(&[String]) -> WaygateResult,
        ),
        (
            "GetLastError",
            get_last_error as fn(&[String]) -> WaygateResult,
        ),
        (
            "MessageBoxA",
            message_box_a as fn(&[String]) -> WaygateResult,
        ),
    ])
}

pub fn create_file_a(_args: &[String]) -> WaygateResult {
    Ok("CreateFileA stub called".to_string())
}

pub fn read_file(_args: &[String]) -> WaygateResult {
    Ok("ReadFile stub called".to_string())
}

pub fn write_file(_args: &[String]) -> WaygateResult {
    Ok("WriteFile stub called".to_string())
}

pub fn close_handle(_args: &[String]) -> WaygateResult {
    Ok("CloseHandle stub called".to_string())
}

pub fn get_last_error(_args: &[String]) -> WaygateResult {
    Ok("GetLastError stub called".to_string())
}

pub fn message_box_a(_args: &[String]) -> WaygateResult {
    Ok("MessageBoxA stub called".to_string())
}
