pub fn hex_string_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    // 移除字符串中可能存在的空格
    let hex = hex.replace(" ", "");

    // 检查字符串长度是否为偶数
    if hex.len() % 2 != 0 {
        return Err("16进制字符串长度必须为偶数".to_string());
    }

    // 每两个字符转换为一个字节
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| format!("无效的16进制字符串: {}", e))
        })
        .collect()
}
