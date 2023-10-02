use super::Handshaker;
use async_trait::async_trait;
use bitcoin::{
    consensus::{deserialize_partial, serialize},
    network::{
        address,
        constants::{self, ServiceFlags},
        message::{self, NetworkMessage, RawNetworkMessage},
        message_network::VersionMessage,
    },
};
use bytes::{Buf, BytesMut};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};
pub struct Btc {
    handshake_complete: bool,
    buffer: BytesMut,
}

impl Btc {
    pub fn new(buff_size: usize) -> Btc {
        Btc {
            handshake_complete: false,
            buffer: BytesMut::with_capacity(buff_size),
        }
    }
}
#[async_trait]
impl Handshaker for Btc {
    async fn startHandshake(
        &self,
        tx_stream: &mut OwnedWriteHalf,
        address: &String,
        user_agent: &String,
    ) -> Result<(), Box<dyn Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        let node_address = SocketAddr::from_str(&address).unwrap();
        let empty_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
        let version_message = VersionMessage::new(
            ServiceFlags::NONE,
            now,
            address::Address::new(&node_address, constants::ServiceFlags::NONE),
            address::Address::new(&empty_address, constants::ServiceFlags::NONE),
            now as u64,
            user_agent.to_string(),
            0,
        );
        let data = serialize(&RawNetworkMessage {
            magic: constants::Network::Bitcoin.magic(),
            payload: NetworkMessage::Version(version_message),
        });
        tx_stream.write_all(data.as_slice()).await?;
        Ok(())
    }
    async fn processHandshakeMsg(
        &mut self,
        message: RawNetworkMessage,
        tx_stream: &mut OwnedWriteHalf,
    ) -> Result<(), Box<dyn Error>> {
        println!(
            "processing message: {} {:?}",
            message.cmd().to_string(),
            message.payload
        );
        match message.payload {
            message::NetworkMessage::Verack => Ok(()),
            message::NetworkMessage::Version(v) => {
                let data = serialize(&RawNetworkMessage {
                    magic: constants::Network::Bitcoin.magic(),
                    payload: NetworkMessage::Verack,
                });
                tx_stream.write_all(data.as_slice()).await?;
                Ok(())
            }
            message::NetworkMessage::Ping(p) => {
                self.handshake_complete = true;
                Ok(())
            }
            _ => Ok(()),
        }
    }
    async fn readTcpStream(
        &mut self,
        stream: &mut OwnedReadHalf,
    ) -> Result<Option<RawNetworkMessage>, Box<dyn Error>> {
        loop {
            if let Ok((message, count)) = deserialize_partial::<RawNetworkMessage>(&self.buffer) {
                println!("read from buffer");
                self.buffer.advance(count);
                return Ok(Some(message));
            }

            if 0 == stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    println!("connection closed by peer");
                    return Err("connection closed by peer".into());
                }
            }
        }
    }
    fn is_handshake_complete(&self) -> bool {
        return self.handshake_complete;
    }
}
