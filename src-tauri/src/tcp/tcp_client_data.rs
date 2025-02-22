use anyhow::Error;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Encoder, FramedRead};

use super::RequestCodec;

pub struct TcpConn {}

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
