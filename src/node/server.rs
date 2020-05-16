use crate::error::DspfsError;
use crate::store::Store;
use log::error;
use log::*;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tokio::sync::mpsc::Receiver;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    // Creates a server struct with the tcplistener
    pub async fn new(addr: impl ToSocketAddrs) -> Result<Self, DspfsError> {
        Ok(Server {
            listener: TcpListener::bind(&addr).await?,
        })
    }

    // Starts listening for requests
    // contains a loop checking for errors
    pub async fn start(mut self, state: Arc<RwLock<dyn Store>>, mut stopper: Receiver<()>) {
        info!("Starting server");
        // Outer loop for catching errors
        tokio::spawn(async move {
            while let Err(e) = self.internal_start(state.clone(), &mut stopper).await {
                error!("an error occurred; error = {:?}", e);
            }
        });
    }

    // Inner loop for receiving messages and calling on [process]
    async fn internal_start(
        &mut self,
        state: Arc<RwLock<dyn Store>>,
        stopper: &mut Receiver<()>,
    ) -> Result<(), DspfsError> {
        info!("Now accepting requests");

        loop {
            select! {
                _ = stopper.recv() => {
                    // If we receive stop signal stop
                    return Ok(())
                }
                accepted = self.listener.accept() => {
                    // Normal message
                    let (stream, addr) = accepted?;
                    let local_store = state.clone();

                    // process the message
                    tokio::spawn(async move {
                        if let Err(e) = process(local_store, stream, addr).await {
                            error!("an error occurred; error = {:?}", e);
                        }
                    });
                }
            }
        }
    }
}

// Actually process the incoming requests
async fn process(
    state: Arc<RwLock<dyn Store>>,
    mut stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), DspfsError> {
    info!("Got a request from {:?}", addr);
    let mut res = Vec::new();



    stream.read_to_end(&mut res).await?;
    info!("Contents: {:?}", String::from_utf8_lossy(&res));

    stream.write(b"Test").await?;

    // Check type of message

    Ok(())
}
