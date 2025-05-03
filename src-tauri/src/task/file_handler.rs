use anyhow::{Context, Result};
use tokio::fs;
use tracing::info;

use crate::{
    model::connect_param::ConnectParam,
    MqttClientData,
    param::read_from_csv_into_struct,
};

/// 加载配置文件
///
/// 从指定路径加载JSON格式的连接配置文件
///
/// # 参数
/// * `file_path` - 配置文件路径
///
/// # 返回
/// 成功返回连接参数对象，失败返回错误
pub async fn load_config_file(file_path: &str) -> Result<ConnectParam> {
    let config_str = fs::read_to_string(file_path)
        .await
        .with_context(|| format!("无法读取配置文件: {}", file_path))?;

    let config = serde_json::from_str(&config_str)
        .with_context(|| "配置文件格式错误")?;

    info!(?config, "加载配置成功");
    Ok(config)
}

/// 写入文件
///
/// 将内容写入指定文件路径
///
/// # 参数
/// * `file_path` - 目标文件路径
/// * `content` - 要写入的文件内容
///
/// # 返回
/// 成功返回Ok，失败返回错误
pub async fn write_file_content(file_path: &str, content: &str) -> Result<()> {
    fs::write(file_path, content)
        .await
        .with_context(|| format!("无法写入文件: {}", file_path))?;
    
    info!("文件写入成功: {}", file_path);
    Ok(())
}

/// 处理客户端CSV文件
///
/// 解析CSV文件中的客户端配置数据
///
/// # 参数
/// * `file_path` - CSV文件路径
///
/// # 返回
/// 成功返回客户端数据列表，失败返回错误
pub async fn process_csv_file(file_path: &str) -> Result<Vec<MqttClientData>> {
    info!("处理CSV文件: {}", file_path);
    let client_data: Vec<MqttClientData> = read_from_csv_into_struct(file_path)
        .await
        .with_context(|| format!("解析CSV文件失败: {}", file_path))?;
    
    info!("CSV解析成功，客户端数量: {}", client_data.len());
    Ok(client_data)
}
