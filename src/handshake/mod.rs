use async_trait::async_trait;
use bitcoin::network::message::RawNetworkMessage;
use std::{error::Error, time::Duration};
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
    sync::{broadcast, mpsc},
};

pub mod btc;
// use crate::types::handshake::HanshakeCompleteEvent;
#[async_trait]
pub trait Handshaker {
    async fn startHandshake(
        &self,
        tx_stream: &mut OwnedWriteHalf,
        address: &String,
        user_agent: &String,
    ) -> Result<(), Box<dyn Error>>;
    async fn processHandshakeMsg(
        &mut self,
        message: RawNetworkMessage,
        tx_stream: &mut OwnedWriteHalf,
    ) -> Result<(), Box<dyn Error>>;
    async fn readTcpStream(
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
        .startHandshake(&mut tx_stream, address, user_agent)
        .await?;
    loop {
        let raw_msg = handshaker.readTcpStream(&mut rx_stream).await?;
        if let Some(msg) = raw_msg {
            handshaker.processHandshakeMsg(msg, &mut tx_stream).await?;
            if handshaker.is_handshake_complete() {
                return Ok(true);
            }
        }
    }
}
// pub async fn handshake<T: Handshaker>(
//     handshaker: &T,
//     timeout: u64,
//     address: String,
// ) -> Result<HanshakeCompleteEvent, Box<dyn Error>> {
//     let (stop_tx, _) = broadcast::channel::<HanshakeCompleteEvent>(1);
//     let mut stop_rx = stop_tx.subscribe();
//     let (handshake_tx, mut handshake_rx) = mpsc::unbounded_channel();
//     let handshake_task = tokio::spawn(async move {
//         loop {
//             select! {
//               Some(msg) = handshake_rx.recv() => {

//               }
//               msg = stop_rx.recv() => {
//                 return msg
//               }
//             }
//         }
//     });
//     let tcp_conn =
//         tokio::time::timeout(Duration::from_millis(timeout), TcpStream::connect(address)).await??;

//     let (tcp_rx, mut tcp_tx) = tcp_conn.into_split();

//     let message_task = tokio::spawn(async move {
//         loop {
//             select! {
//               Some(msg) = tcp_rx.recv()
//             }
//         }
//     });

//     Ok(())
// }
