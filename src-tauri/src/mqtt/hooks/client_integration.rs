use std::sync::Arc;

use rumqttc::{Event, Packet};
use tokio::sync::OnceCell;
use tracing::{debug, info};

use super::{MqttHookManager, MqttHookProcessor, MqttHookResult};
use crate::mqtt::{MqttClient, MqttHookContext};
use crate::{context::get_app_state, ConnectionState};
use async_trait::async_trait;

/// 全局MQTT钩子管理器单例
static MQTT_HOOK_MANAGER: OnceCell<MqttHookManager> = OnceCell::const_new();

/// 获取全局MQTT钩子管理器实例
pub async fn get_mqtt_hook_manager() -> &'static MqttHookManager {
    MQTT_HOOK_MANAGER
        .get_or_init(|| async { MqttHookManager::new() })
        .await
}

/// 处理MQTT事件
///
/// 将事件路由到已注册的钩子处理器
///
/// # 参数
/// * `event` - MQTT事件
pub async fn process_event(event: &Event, client_id: String) {
    // 获取钩子管理器实例
    let hook_manager = get_mqtt_hook_manager().await;

    // 创建上下文并处理事件
    if let Event::Incoming(Packet::Publish(publish)) = event {
        let topic = publish.topic.clone();
        let payload = publish.payload.clone();

        debug!("处理MQTT发布事件，主题: {}", topic);

        // 创建处理上下文
        let mut context =
            MqttHookContext::new(Packet::Publish(publish.clone()), topic, payload.to_vec());
        context.add_metadata("client_id", &client_id);

        // 处理消息
        if hook_manager.process_message(context).await {
            debug!("MQTT事件已被钩子系统处理");
        } else {
            debug!("没有钩子处理器处理该MQTT事件");
        }
    } else {
        // 对于非Publish事件，创建只有packet的上下文
        let mut context = MqttHookContext::empty();
        match event {
            Event::Incoming(packet) => {
                context.packet = Some(packet.clone());
            }
            _ => {}
        }
        context.add_metadata("client_id", &client_id);

        // 让钩子管理器处理事件
        if hook_manager.process_message(context).await {
            debug!("MQTT事件已被钩子系统处理");
        } else {
            debug!("没有钩子处理器处理该MQTT事件");
        }
    }
}


/// 连接确认(ConnAck)钩子处理器
pub struct ConnAckProcessor {
    client: Arc<MqttClient>,
}

impl ConnAckProcessor {
    pub fn new(client: Arc<MqttClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MqttHookProcessor for ConnAckProcessor {
    fn name(&self) -> &str {
        "connack_processor"
    }

    fn priority(&self) -> i32 {
        // 高优先级处理连接事件
        10
    }

    fn matches_topic(&self, _topic: &str) -> bool {
        // ConnAck 不关联特定主题
        true
    }

    async fn handle(&self, context: MqttHookContext) -> MqttHookResult {
        // 只处理 ConnAck 包
        if let Some(Packet::ConnAck(_)) = context.get_packet() {
            let app_state = get_app_state();

            // 从上下文中提取客户端ID
            if let Some(client_id) = context.get_metadata("client_id") {
                if let Some(mut client) = app_state.mqtt_clients().get_mut(client_id) {
                    client.set_connection_state(ConnectionState::Connected);
                }
                // 标记事件已处理
                return MqttHookResult::Terminate(context);
            }
        }

        // 不是我们关心的事件，继续处理链
        MqttHookResult::Continue(context)
    }
}


/// 初始化MQTT钩子系统
pub async fn init_mqtt_hooks(mqtt_client: MqttClient) {
    // 确保钩子管理器已初始化
    let hook_manager = get_mqtt_hook_manager().await;
    let connack_processor = Arc::new(ConnAckProcessor::new(mqtt_client.into()));
    hook_manager.register(connack_processor);
    info!("MQTT钩子系统已初始化");
}