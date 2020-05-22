use crate::message::Message;
use crate::stream::encryptedstream::EncryptedStream;
use crate::user::PrivateUser;
use anyhow::{Context, Result};
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Client {
    stream: EncryptedStream<TcpStream>,
}

impl Client {
    pub async fn new(addr: impl ToSocketAddrs, user: &PrivateUser) -> Result<Self> {
        let tcpstream = TcpStream::connect(addr)
            .await
            .context("failed to create tcp connection")?;

        Ok(Self {
            stream: EncryptedStream::initiator(tcpstream, &user)
                .await
                .context("failed to initiate secure tunnel")?,
        })
    }

    pub async fn send(&mut self, msg: Message) -> Result<()> {
        self.stream.send_message(msg).await
    }

    pub async fn recv(&mut self, limit: usize) -> Result<Message> {
        self.stream.recv_message(limit).await
    }
}
