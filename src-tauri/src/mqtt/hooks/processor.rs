use std::{
    fmt::{self, Debug, Formatter},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use async_trait::async_trait;

use super::types::{MqttHookContext, MqttHookResult};

/// MQTT钩子处理器
///
/// 定义MQTT消息处理钩子的行为
#[async_trait]
pub trait MqttHookProcessor: Send + Sync + 'static {
    /// 处理器名称
    fn name(&self) -> &str;

    /// 处理器优先级（数字越小优先级越高）
    fn priority(&self) -> i32;

    /// 检查处理器是否应处理给定主题的消息
    fn matches_topic(&self, topic: &str) -> bool;

    /// 处理MQTT消息
    ///
    /// # 参数
    /// * `context` - MQTT消息上下文
    ///
    /// # 返回
    /// 返回处理结果
    async fn handle(&self, context: MqttHookContext) -> MqttHookResult;

    /// 消息处理后的后续操作（可选实现）
    ///
    /// 无论处理结果如何，后处理逻辑都会执行
    ///
    /// # 参数
    /// * `context` - MQTT消息上下文
    /// * `result` - 处理结果
    async fn post_handle(&self, context: MqttHookContext, result: &MqttHookResult) {
        // 默认实现为空，派生类可以覆盖
        let _ = (context, result);
    }
}

/// 基本的MQTT钩子处理器实现
pub struct MqttHookProcessorImpl {
    /// 处理器名称
    name: String,
    /// 处理器优先级
    priority: i32,
    /// 匹配的主题
    topic: String,
    /// 消息处理函数
    handler: Arc<
        dyn Fn(MqttHookContext) -> Pin<Box<dyn Future<Output = MqttHookResult> + Send>>
            + Send
            + Sync,
    >,
    /// 后处理函数（可选）
    post_handler: Option<
        Arc<
            dyn Fn(MqttHookContext, &MqttHookResult) -> Pin<Box<dyn Future<Output = ()> + Send>>
                + Send
                + Sync,
        >,
    >,
}

impl Debug for MqttHookProcessorImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MqttHookProcessorImpl")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("topic", &self.topic)
            .finish()
    }
}

#[async_trait]
impl MqttHookProcessor for MqttHookProcessorImpl {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn matches_topic(&self, topic: &str) -> bool {
        self.topic == topic
    }

    async fn handle(&self, context: MqttHookContext) -> MqttHookResult {
        (self.handler)(context).await
    }

    async fn post_handle(&self, context: MqttHookContext, result: &MqttHookResult) {
        if let Some(post_handler) = &self.post_handler {
            post_handler(context, result).await;
        }
    }
}

/// MQTT钩子处理器构建器
///
/// 使用构建器模式创建钩子处理器
pub struct MqttHookProcessorBuilder {
    name: String,
    priority: i32,
    topic: Option<String>,
    handler: Option<
        Arc<
            dyn Fn(MqttHookContext) -> Pin<Box<dyn Future<Output = MqttHookResult> + Send>>
                + Send
                + Sync,
        >,
    >,
    post_handler: Option<
        Arc<
            dyn Fn(MqttHookContext, &MqttHookResult) -> Pin<Box<dyn Future<Output = ()> + Send>>
                + Send
                + Sync,
        >,
    >,
}

impl Default for MqttHookProcessorBuilder {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            priority: 100, // 默认中等优先级
            topic: None,
            handler: None,
            post_handler: None,
        }
    }
}

impl MqttHookProcessorBuilder {
    /// 创建新的构建器实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置处理器名称
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// 设置处理器优先级
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 设置要匹配的主题
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// 设置处理函数
    pub fn handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(MqttHookContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = MqttHookResult> + Send + 'static,
    {
        self.handler = Some(Arc::new(move |ctx| Box::pin(handler(ctx))));
        self
    }

    /// 设置后处理函数
    pub fn post_handler<F, Fut>(mut self, post_handler: F) -> Self
    where
        F: Fn(MqttHookContext, &MqttHookResult) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.post_handler = Some(Arc::new(move |ctx, result| {
            Box::pin(post_handler(ctx, result))
        }));
        self
    }

    /// 构建钩子处理器
    pub fn build(self) -> Result<Arc<dyn MqttHookProcessor>, String> {
        // 验证必要参数
        let topic = self.topic.ok_or_else(|| "Topic is required".to_string())?;
        let handler = self
            .handler
            .ok_or_else(|| "Handler is required".to_string())?;

        Ok(Arc::new(MqttHookProcessorImpl {
            name: self.name,
            priority: self.priority,
            topic,
            handler,
            post_handler: self.post_handler,
        }))
    }
}
