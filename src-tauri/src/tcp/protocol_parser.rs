use anyhow::Error;
use tokio_util::codec::{Decoder, Encoder};

use super::modbus::ModbusFrame;
#[derive(Debug)]
pub struct RequestData {
    data: String,
}
// 定义协议类型枚举
#[derive(Debug, Clone)]
pub enum ProtocolType {
    ModBus,
    Custom,
}

// 定义协议解析trait
pub trait ProtocolParser {
    fn parse_request(&self, data: &[u8]) -> Result<RequestData, Error>;
    fn encode_response(&self, response: &ResponseData) -> Result<Vec<u8>, Error>;
}

// Modbus协议解析器实现
pub struct ModbusParser;
impl ProtocolParser for ModbusParser {
    fn parse_request(&self, data: &[u8]) -> Result<RequestData, Error> {
        // 实现Modbus协议解析逻辑
        let mut frame = ModbusFrame::new();
        frame.parse_frame(data)?;
        Ok(RequestData {
            data: format!("{:?}", frame),
        })
    }

    fn encode_response(&self, response: &ResponseData) -> Result<Vec<u8>, Error> {
        // 实现Modbus响应编码逻辑
        Ok(response.data.as_bytes().to_vec())
    }
}

pub struct CustomParser;
impl ProtocolParser for CustomParser {
    fn parse_request(&self, data: &[u8]) -> Result<RequestData, Error> {
        Ok(RequestData {
            data: format!("{:?}", data),
        })
    }

    fn encode_response(&self, response: &ResponseData) -> Result<Vec<u8>, Error> {
        todo!()
    }
}

pub struct RequestCodec;
impl RequestCodec {
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

        // 这里假设第一个字节表示协议类型
        let protocol_type = match data[0] {
            0x01 => ProtocolType::ModBus,
            0x02 => ProtocolType::Custom,
            _ => return Err(Error::msg("协议类型错误")),
        };

        // 获取对应的协议解析器
        let parser = Self::get_protocol(protocol_type);
        // 解析数据
        Ok(Some(parser.parse_request(&data[1..])?))
    }
}

#[derive(Debug)]
pub struct ResponseData {
    data: String,
}

pub struct ResponseCodec;

impl Encoder<ResponseData> for ResponseCodec {
    type Error = Error;

    fn encode(&mut self, item: ResponseData, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}
