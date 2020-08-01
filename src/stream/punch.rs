use tokio::net::{TcpStream, ToSocketAddrs, TcpListener};
use tokio::io;
use std::ops::Deref;

pub struct HolepunchingTcpStream {
    stream: TcpStream,
}

impl HolepunchingTcpStream {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<HolepunchingTcpStream> {
        Ok(HolepunchingTcpStream {
            stream: TcpStream::connect(addr).await?,
        })
    }

    pub fn from_std(stream: std::net::TcpStream) -> io::Result<HolepunchingTcpStream> {
        Ok(HolepunchingTcpStream {
            stream: TcpStream::from_std(stream)?,
        })
    }

    pub fn from_tokio(stream: tokio::net::TcpStream) -> io::Result<HolepunchingTcpStream> {
        Ok(HolepunchingTcpStream {
            stream,
        })
    }

    pub fn punch_hole(&self) {

    }
}

impl Deref for HolepunchingTcpStream {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}


pub struct HolepunchingTcpListener {
    listener: TcpListener,
}

impl HolepunchingTcpListener {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<HolepunchingTcpListener> {
        Ok(HolepunchingTcpListener {
            listener: TcpListener::bind(addr).await?
        })
    }

    pub fn from_std(listener: std::net::TcpListener) -> io::Result<HolepunchingTcpListener> {
        Ok(HolepunchingTcpListener {
            listener: TcpListener::from_std(listener)?,
        })
    }

    pub fn from_tokio(listener: TcpListener) -> io::Result<HolepunchingTcpListener> {
        Ok(HolepunchingTcpListener {
            listener,
        })
    }

}
