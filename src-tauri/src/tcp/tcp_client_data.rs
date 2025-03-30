use anyhow::Error;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Encoder, FramedRead};

use super::RequestCodec;

/// TCP连接结构体
pub struct TcpConn {}

/// 响应数据结构
/// 
/// 包含TCP响应的数据内容
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

/// 处理TCP客户端连接
/// 
/// 读取并解析来自客户端的数据
/// 
/// # 参数
/// * `tcp_stream` - TCP连接流
async fn process_client(tcp_stream: TcpStream) {
    let (client_reader, _) = tcp_stream.into_split();

    let mut frame_reader = FramedRead::new(client_reader, RequestCodec);

    loop {
        match frame_reader.next().await {
            None => {
                break;
            }
            Some(Err(_e)) => {
                break;
            }
            Some(Ok(req_resp)) => {
                println!("Received request: {:?}", req_resp);
            }
        }
    }
}
