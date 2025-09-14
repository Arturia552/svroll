use anyhow::Error;
use tokio_util::codec::{Decoder, Encoder};

use super::modbus::ModbusFrame;


/// 请求编解码器
pub struct RequestCodec;

impl Decoder for RequestCodec {
    type Item = ModbusFrame;
    type Error = Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 如果没有数据，返回None等待更多数据
        if src.is_empty() {
            return Ok(None);
        }

        let mut frame = ModbusFrame::new();
        match frame.parse_frame(src)? {
            true => Ok(Some(frame)), // 成功解析完整帧
            false => Ok(None),       // 数据不足，等待更多数据
        }
    }
}


/// 响应编解码器
pub struct ResponseCodec;

impl Encoder<ModbusFrame> for ResponseCodec {
    type Error = Error;

    fn encode(&mut self, item: ModbusFrame, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let encoded = item.encode_frame();
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}
