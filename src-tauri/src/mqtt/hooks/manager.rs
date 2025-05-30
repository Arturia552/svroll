use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tracing::debug;

use super::{
    processor::MqttHookProcessor,
    types::{MqttHookContext, MqttHookResult},
};

/// MQTT钩子管理器
///
/// 管理和协调多个MQTT消息处理钩子
#[derive(Default, Clone)]
pub struct MqttHookManager {
    /// 钩子处理器集合，按名称索引
    processors: Arc<RwLock<HashMap<String, Arc<dyn MqttHookProcessor>>>>,
    /// 处理器优先级缓存，避免每次处理消息时重新计算
    priority_cache: Arc<RwLock<Option<Vec<Arc<dyn MqttHookProcessor>>>>>,
}

impl MqttHookManager {
    /// 创建新的钩子管理器实例
    pub fn new() -> Self {
        Self {
            processors: Arc::new(RwLock::new(HashMap::new())),
            priority_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// 注册新的钩子处理器
    ///
    /// # 参数
    /// * `processor` - 实现了MqttHookProcessor的处理器
    ///
    /// # 返回
    /// 返回自身，支持链式调用
    pub async fn register(&self, processor: Arc<dyn MqttHookProcessor>) -> &Self {
        let name = processor.name().to_string();
        debug!("注册MQTT钩子处理器: {}", name);

        let mut processors = self.processors.write().await;
        processors.insert(name, processor);

        // 清除优先级缓存，强制下次处理消息时重建
        let mut cache = self.priority_cache.write().await;
        *cache = None;

        self
    }

    /// 注销钩子处理器
    ///
    /// # 参数
    /// * `name` - 处理器名称
    ///
    /// # 返回
    /// 如果找到并移除处理器则返回true，否则返回false
    pub async fn unregister(&self, name: &str) -> bool {
        let mut processors = self.processors.write().await;
        let result = processors.remove(name).is_some();

        if result {
            // 清除优先级缓存
            let mut cache = self.priority_cache.write().await;
            *cache = None;
            debug!("已注销MQTT钩子处理器: {}", name);
        }

        result
    }

    /// 清空所有注册的处理器
    pub async fn clear(&self) {
        let mut processors = self.processors.write().await;
        processors.clear();

        let mut cache = self.priority_cache.write().await;
        *cache = None;

        debug!("已清空所有MQTT钩子处理器");
    }

    /// 获取排序后的处理器列表
    ///
    /// 根据优先级排序，缓存结果以提高性能
    async fn get_sorted_processors(&self) -> Vec<Arc<dyn MqttHookProcessor>> {
        // 先检查缓存
        if let Some(cached) = self.priority_cache.read().await.as_ref() {
            return cached.clone();
        }

        // 缓存未命中，构建排序列表
        let processors = self.processors.read().await;
        let mut sorted_processors: Vec<_> = processors.values().cloned().collect();

        // 按优先级排序（优先级数字小的排在前面）
        sorted_processors.sort_by_key(|p| p.priority());

        // 更新缓存
        let mut cache = self.priority_cache.write().await;
        *cache = Some(sorted_processors.clone());

        sorted_processors
    }

    /// 处理MQTT消息
    ///
    /// # 参数
    /// * `context` - MQTT消息上下文
    ///
    /// # 返回
    /// 返回是否有处理器处理了该消息
    pub async fn process_message(&self, initial_context: MqttHookContext) -> bool {
        let sorted_processors = self.get_sorted_processors().await;

        let topic = initial_context
            .topic
            .clone()
            .unwrap_or_else(|| "".to_owned());

        let matching_processors: Vec<Arc<dyn MqttHookProcessor>> = sorted_processors
            .into_iter()
            .filter(|p| p.matches_topic(&topic))
            .collect::<Vec<_>>();

        if matching_processors.is_empty() {
            if let Some(topic) = initial_context.get_topic() {
                debug!("没有匹配主题的处理器: {}", topic);
            } else {
                debug!("没有匹配无主题消息的处理器");
            }
            return false;
        }

        debug!(
            "找到{}个匹配消息的处理器",
            matching_processors.len()
        );

        // 链式执行所有匹配的处理器
        let mut current_context = initial_context;

        for processor in matching_processors {
            debug!("执行处理器: {}", processor.name());

            // 执行处理函数
            let result = processor.handle(current_context.clone()).await;

            // 执行后处理逻辑
            processor
                .post_handle(current_context.clone(), &result)
                .await;

            // 根据处理结果决定是否继续处理链
            match result {
                MqttHookResult::Continue(new_context) => {
                    // 继续处理，更新上下文
                    current_context = new_context;
                }
                MqttHookResult::Terminate(_) => {
                    // 终止处理链，但仍算作成功处理
                    debug!(
                        "处理器 {} 终止了处理链",
                        processor.name()
                    );
                    break;
                }
                MqttHookResult::Ignore => {
                    // 忽略当前消息
                    debug!("处理器 {} 忽略了该消息", processor.name());
                    return true; // 仍然返回true，因为有匹配的处理器
                }
            }
        }

        true // 简化逻辑，有匹配处理器就返回true
    }
}
