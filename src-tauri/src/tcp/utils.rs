/// 将十六进制字符串转换为字节数组
/// 
/// 支持带空格或不带空格的十六进制字符串
/// 
/// # 参数
/// * `hex` - 要转换的十六进制字符串
///
/// # 返回
/// 成功返回字节数组，失败返回错误信息
pub fn hex_string_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.replace(" ", "");

    if hex.len() % 2 != 0 {
        return Err("16进制字符串长度必须为偶数".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| format!("无效的16进制字符串: {}", e))
        })
        .collect()
}
