use crate::node::error::NodeError;
use crate::node::state::State;
use log::error;
use log::*;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tokio::sync::mpsc::Receiver;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    // Creates a server struct with the tcplistener
    pub async fn new(addr: impl ToSocketAddrs) -> Result<Self, NodeError> {
        Ok(Server {
            listener: TcpListener::bind(&addr).await?,
        })
    }

    // Starts listening for requests
    // contains a loop checking for errors
    pub async fn start(mut self, state: Arc<Mutex<State>>, mut stopper: Receiver<()>) {
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
        state: Arc<Mutex<State>>,
        stopper: &mut Receiver<()>,
    ) -> Result<(), NodeError> {
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
                    let local_state = state.clone();

                    // process the message
                    tokio::spawn(async move {
                        if let Err(e) = Self::process(local_state, stream, addr).await {
                            error!("an error occurred; error = {:?}", e);
                        }
                    });
                }
            }
        }
    }

    // Actually process the incoming requests
    async fn process(
        _state: Arc<Mutex<State>>,
        mut stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), NodeError> {
        info!("Got a request from {:?}", addr);
        let mut res = Vec::new();

        stream.read_to_end(&mut res).await?;

        // TODO: Actually do something instead of printing
        info!("Contents: {:?}", String::from_utf8_lossy(&res));

        Ok(())
    }
}
