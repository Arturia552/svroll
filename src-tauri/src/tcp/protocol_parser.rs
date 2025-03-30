use anyhow::Error;
use tokio_util::codec::{Decoder, Encoder};

use super::modbus::ModbusFrame;

/// TCP请求数据结构
#[derive(Debug)]
pub struct RequestData {
    /// 请求数据内容
    data: String,
}

/// 协议类型枚举
/// 
/// 定义系统支持的协议类型
#[derive(Debug, Clone)]
pub enum ProtocolType {
    /// Modbus协议
    ModBus,
    /// 自定义协议
    Custom,
}

/// 协议解析器特性
/// 
/// 定义协议解析器需要实现的方法
pub trait ProtocolParser {
    /// 解析请求数据
    /// 
    /// # 参数
    /// * `data` - 原始字节数据
    /// 
    /// # 返回
    /// 成功返回解析后的请求数据，失败返回错误
    fn parse_request(&self, data: &[u8]) -> Result<RequestData, Error>;
    
    /// 编码响应数据
    /// 
    /// # 参数
    /// * `response` - 要编码的响应数据
    /// 
    /// # 返回
    /// 成功返回编码后的字节数组，失败返回错误
    fn encode_response(&self, _response: &ResponseData) -> Result<Vec<u8>, Error>;
}

/// Modbus协议解析器实现
pub struct ModbusParser;
impl ProtocolParser for ModbusParser {
    fn parse_request(&self, data: &[u8]) -> Result<RequestData, Error> {
        let mut frame = ModbusFrame::new();
        frame.parse_frame(data)?;
        Ok(RequestData {
            data: format!("{:?}", frame),
        })
    }

    fn encode_response(&self, response: &ResponseData) -> Result<Vec<u8>, Error> {
        Ok(response.data.as_bytes().to_vec())
    }
}

/// 自定义协议解析器实现
pub struct CustomParser;
impl ProtocolParser for CustomParser {
    fn parse_request(&self, data: &[u8]) -> Result<RequestData, Error> {
        Ok(RequestData {
            data: format!("{:?}", data),
        })
    }

    fn encode_response(&self, _response: &ResponseData) -> Result<Vec<u8>, Error> {
        todo!()
    }
}

/// 请求编解码器
/// 
/// 用于解析TCP数据帧
pub struct RequestCodec;
impl RequestCodec {
    /// 根据协议类型获取相应的协议解析器
    /// 
    /// # 参数
    /// * `protocol_type` - 协议类型
    /// 
    /// # 返回
    /// 返回对应的协议解析器实现
    pub fn get_protocol(protocol_type: ProtocolType) -> Box<dyn ProtocolParser> {
        let parser: Box<dyn ProtocolParser> = match protocol_type {
            ProtocolType::Custom => Box::new(CustomParser),
            ProtocolType::ModBus => Box::new(ModbusParser),
        };
        parser
    }
}

impl Decoder for RequestCodec {
    type Item = RequestData;
    type Error = Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let len = src.len();
        let data = src.split_to(len);

        let protocol_type = match data[0] {
            0x01 => ProtocolType::ModBus,
            0x02 => ProtocolType::Custom,
            _ => return Err(Error::msg("协议类型错误")),
        };

        let parser = Self::get_protocol(protocol_type);
        Ok(Some(parser.parse_request(&data[1..])?))
    }
}

/// 响应数据结构
/// 
/// 包含要发送的响应数据
#[derive(Debug)]
pub struct ResponseData {
    /// 响应内容
    data: String,
}

/// 响应编解码器
/// 
/// 用于将ResponseData编码为字节流
pub struct ResponseCodec;

impl Encoder<ResponseData> for ResponseCodec {
    type Error = Error;

    fn encode(&mut self, _item: ResponseData, _dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}
