use anyhow::{Context, Result};
use tracing::info;

use crate::model::Rs2JsEntity;
use crate::Rs2JsMsgType;

/// 记录日志并发送通知
///
/// 将消息记录到日志并发送到前端显示
///
/// # 参数
/// * `tx` - 消息发送通道
/// * `msg_type` - 消息类型
/// * `msg` - 消息内容
///
/// # 返回
/// 成功返回Ok，失败返回错误
pub async fn log_and_notify(
    tx: &tauri::async_runtime::Sender<Rs2JsEntity>,
    msg_type: Rs2JsMsgType,
    msg: &str,
) -> Result<()> {
    info!("{}", msg);
    tx.send(Rs2JsEntity::new(msg_type, msg.to_string()))
        .await
        .context(format!("发送消息失败: {}", msg))
}

/// 格式化错误消息
///
/// 将错误信息格式化成友好的消息字符串
///
/// # 参数
/// * `context` - 错误上下文
/// * `error` - 错误对象
///
/// # 返回
/// 格式化的错误消息
pub fn format_error(context: &str, error: &anyhow::Error) -> String {
    format!("{}: {}", context, error)
}

/// 记录并格式化错误
///
/// 记录错误并返回格式化的错误消息字符串
///
/// # 参数
/// * `context` - 错误上下文
/// * `error` - 错误对象
///
/// # 返回
/// 格式化的错误消息
pub fn log_error(context: &str, error: &anyhow::Error) -> String {
    let error_msg = format_error(context, error);
    tracing::error!("{}", error_msg);
    error_msg
}
