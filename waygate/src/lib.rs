pub type WaygateResult = Result<String, String>;

pub fn dispatch(symbol: &str, args: &[String]) -> WaygateResult {
    match symbol {
        "CreateFileA" => create_file_a(args),
        "ReadFile" => read_file(args),
        "WriteFile" => write_file(args),
        "CloseHandle" => close_handle(args),
        "GetLastError" => get_last_error(args),
        "MessageBoxA" => message_box_a(args),
        _ => Err(format!("waygate: symbol '{symbol}' is not implemented")),
    }
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
