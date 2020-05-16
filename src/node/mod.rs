use crate::error::DspfsError;
use crate::message::Message;
use crate::node::server::Server;
use crate::user::PublicUser;
use std::mem;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::{channel, Sender};

mod server;
mod state;

enum ServerState {
    Started(Sender<()>),
    Unstarted(Server),
    Stopped,
}

pub struct Node {
    /// This option becomes None once the server is started.
    server: ServerState,
    state: Arc<Mutex<state::State>>,
}

impl Node {
    // new server
    pub async fn new(addr: impl ToSocketAddrs) -> Result<Self, DspfsError> {
        let server = Server::new(addr).await?;

        Ok(Self {
            server: ServerState::Unstarted(server),
            state: Arc::new(Mutex::new(state::State::new())),
        })
    }

    pub async fn send_hello_world(&self, addr: impl ToSocketAddrs) -> Result<(), DspfsError> {
        // just sends hello world to an address for now.

        let mut sock = TcpStream::connect(addr).await?;

        sock.write_all(b"hello world!").await?;

        Ok(())
    }

    pub async fn send(&self, _to: PublicUser, _message: impl Message) -> Result<(), DspfsError> {
        todo!()
    }

    /// Stops the running dspfs server.
    pub async fn stop(&mut self) -> Result<(), DspfsError> {
        match &mut self.server {
            s @ ServerState::Started(_) => {
                if let ServerState::Started(mut stopper) = mem::replace(s, ServerState::Stopped) {
                    if stopper.send(()).await.is_err() {
                        Err("Could not stop server due to channel failure.".into())
                    } else {
                        Ok(())
                    }
                } else {
                    unreachable!()
                }
            }
            ServerState::Unstarted(_) => {
                Err("Can't stop server because it hasn't yet been started".into())
            }
            ServerState::Stopped => Err("Can't stop server because already stopped".into()),
        }
    }

    /// Starts the dspfs server.
    pub async fn start_server(&mut self) -> Result<(), DspfsError> {
        match &mut self.server {
            ServerState::Started(_) => {
                Err("Couldn't start server because it was already started.".into())
            },
            s @ ServerState::Unstarted(_) => {
                // This creates a channel which upon receiving something will stop the server.
                let (tx, rx) = channel(2);

                if let ServerState::Unstarted(server) = mem::replace(s, ServerState::Started(tx)) {
                    server.start(self.state.clone(), rx).await;

                    Ok(())
                } else {
                    // Unreachable because we have already matched tobe sure thathe type is Unstarted
                    unreachable!()
                }
            },
            ServerState::Stopped => {
                Err("Couldn't start server because it was already stopped. A server may only be started once.".into())
            },
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::init;
    use crate::node::Node;
    use tokio::time::{delay_for, Duration};

    #[tokio::test]
    pub async fn test_simple_stream() {
        init();

        let mut n1 = Node::new("0.0.0.0:8123").await.unwrap();
        let mut n2 = Node::new("0.0.0.0:8124").await.unwrap();

        n1.start_server().await.unwrap();
        n2.start_server().await.unwrap();

        delay_for(Duration::from_secs_f64(0.5)).await;

        n1.send_hello_world("localhost:8124").await.unwrap();

        delay_for(Duration::from_secs_f64(0.5)).await;

        n1.stop().await.unwrap();
        n2.stop().await.unwrap();
    }
}
