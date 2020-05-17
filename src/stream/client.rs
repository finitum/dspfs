use crate::error::DspfsError;
use crate::message::Message;
use crate::stream::encryptedstream::EncryptedStream;
use crate::user::PrivateUser;
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Client {
    stream: EncryptedStream<TcpStream>,
}

impl Client {
    pub async fn new(addr: impl ToSocketAddrs, user: &PrivateUser) -> Result<Self, DspfsError> {
        let tcpstream = TcpStream::connect(addr).await?;

        Ok(Self {
            stream: EncryptedStream::initiator(tcpstream, &user).await?,
        })
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), DspfsError> {
        self.stream.send_message(msg).await
    }

    pub async fn recv(&mut self, limit: usize) -> Result<Message, DspfsError> {
        self.stream.recv_message(limit).await
    }
}
