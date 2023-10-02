use async_trait::async_trait;
use bitcoin::network::message::RawNetworkMessage;
use std::{error::Error, time::Duration};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};

pub mod btc;
// use crate::types::handshake::HanshakeCompleteEvent;
#[async_trait]
pub trait Handshaker {
    async fn start_handshake(
        &self,
        tx_stream: &mut OwnedWriteHalf,
        address: &String,
        user_agent: &String,
    ) -> Result<(), Box<dyn Error>>;
    async fn process_handshake_msg(
        &mut self,
        message: RawNetworkMessage,
        tx_stream: &mut OwnedWriteHalf,
    ) -> Result<(), Box<dyn Error>>;
    async fn read_tcp_stream(
        &mut self,
        stream: &mut OwnedReadHalf,
    ) -> Result<Option<RawNetworkMessage>, Box<dyn Error>>;
    fn is_handshake_complete(&self) -> bool;
}

pub async fn handshake<T: Handshaker>(
    handshaker: &mut T,
    tcp_timeout: u64,
    handshake_timeout: u64,
    address: &String,
    user_agent: &String,
) -> Result<bool, Box<dyn Error>> {
    return tokio::time::timeout(
        Duration::from_millis(handshake_timeout),
        handshake_util(handshaker, tcp_timeout, address, user_agent),
    )
    .await?;
}

async fn handshake_util<T: Handshaker>(
    handshaker: &mut T,
    tcp_timeout: u64,
    address: &String,
    user_agent: &String,
) -> Result<bool, Box<dyn Error>> {
    let tcp_stream = tokio::time::timeout(
        Duration::from_millis(tcp_timeout),
        TcpStream::connect(address.clone()),
    )
    .await??;
    println!("tcp connection established with peer");
    let (mut rx_stream, mut tx_stream) = tcp_stream.into_split();
    handshaker
        .start_handshake(&mut tx_stream, address, user_agent)
        .await?;
    loop {
        let raw_msg = handshaker.read_tcp_stream(&mut rx_stream).await?;
        if let Some(msg) = raw_msg {
            handshaker
                .process_handshake_msg(msg, &mut tx_stream)
                .await?;
            if handshaker.is_handshake_complete() {
                return Ok(true);
            }
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::CONFIG;
    use btc::Btc;

    #[tokio::test]
    async fn test_handshake_timeout() {
        let mut handshaker = Btc::new(1024);
        tokio::time::timeout(
            Duration::from_millis(200),
            handshake(
                &mut handshaker,
                100,
                100,
                &"0.0.0.0:80".to_string(),
                &CONFIG.btc.useragent,
            ),
        )
        .await
        .unwrap()
        .expect_err("should timeout");
    }
}
