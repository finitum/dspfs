use crate::node::error::NodeError;
use crate::node::state::State;
use log::error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::Receiver;
use tokio::select;
use log::*;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new(addr: impl ToSocketAddrs) -> Result<Self, NodeError> {
        Ok(Server {
            listener: TcpListener::bind(&addr).await?,
        })
    }

    pub async fn start(mut self, state: Arc<Mutex<State>>, mut stopper: Receiver<()>) {
        info!("Starting server");
        tokio::spawn(async move {
            while let Err(e) = self.internal_start(state.clone(), &mut stopper).await {
                error!("an error occurred; error = {:?}", e);
            }
        });
    }

    async fn internal_start(&mut self, state: Arc<Mutex<State>>,  stopper: &mut Receiver<()>) -> Result<(), NodeError> {
        info!("Now accepting requests");

        loop {
            select! {
                _ = stopper.recv() => {
                    return Ok(())
                }
                accepted = self.listener.accept() => {
                    let (stream, addr) = accepted?;
                    let local_state = state.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::process(local_state, stream, addr).await {
                            error!("an error occurred; error = {:?}", e);
                        }
                    });
                }
            }
        }
    }

    async fn process(
        state: Arc<Mutex<State>>,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), NodeError> {
        info!("Got a request from {:?}", addr);
        Ok(())
    }
}
