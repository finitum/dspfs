use crate::error::DspfsError;
use crate::store::SharedStore;
use crate::stream::encryptedstream::EncryptedStream;
use crate::user::PrivateUser;
use log::error;
use log::*;
use std::net::SocketAddr;
use std::ops::Deref;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct Server {
    listener: TcpListener,
    store: SharedStore,
}

pub struct ServerHandle {
    pub(self) stop_channel: Sender<()>,
}

impl ServerHandle {
    pub async fn stop(mut self) -> Result<(), DspfsError> {
        self.stop_channel.send(()).await?;
        Ok(())
    }
}

impl Server {
    // Creates a server struct with the tcplistener
    pub async fn new(addr: impl ToSocketAddrs, store: SharedStore) -> Result<Self, DspfsError> {
        Ok(Server {
            listener: TcpListener::bind(&addr).await?,
            store: store.clone(),
        })
    }

    // Starts listening for requests
    // contains a loop checking for errors
    pub async fn start(mut self) -> ServerHandle {
        let (tx, mut rx) = channel(2);

        info!("Starting server");
        // Outer loop for catching errors
        tokio::spawn(async move {
            while let Err(e) = self.internal_start(&mut rx).await {
                error!("an error occurred; error = {:?}", e);
            }
        });

        ServerHandle { stop_channel: tx }
    }

    // Inner loop for receiving messages and calling on [process]
    async fn internal_start(&mut self, stopper: &mut Receiver<()>) -> Result<(), DspfsError> {
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
                    let local_store = self.store.clone();

                    // process the message
                    tokio::spawn(async move {
                        if let Err(e) = receive(local_store, stream, addr).await {
                            error!("an error occurred; error = {:?}", e);
                        }
                    });
                }
            }
        }
    }
}

// Actually process the incoming requests
async fn receive(
    store: SharedStore,
    stream: TcpStream,
    _addr: SocketAddr,
) -> Result<(), DspfsError> {
    let user = {
        let guard = store.read().await;

        // FIXME
        PrivateUser::load_from_store(guard.deref().deref())?
    };
    let mut es = EncryptedStream::receiver(stream, user).await?;

    while let Ok(message) = es.recv_message(4096).await {
        info!("{:?}", message);
    }

    // TODO: Check type of message

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use crate::init;
    use crate::message::Message;
    use crate::store::inmemory::InMemory;
    use crate::store::Store;
    use crate::stream::{Client, Server};
    use crate::user::PrivateUser;
    use tokio::time::{delay_for, Duration};

    #[tokio::test]
    pub async fn test_simple_stream() {
        init();

        let (u1, doc1) = PrivateUser::new("Test1").unwrap();
        let (u2, _) = PrivateUser::new("Test2").unwrap();

        let store1 = InMemory::default().shared();

        store1.write().await.set_signing_key(doc1);
        store1.write().await.set_self_user(u1.to_owned());

        let server = Server::new("0.0.0.0:8123", store1)
            .await
            .unwrap()
            .start()
            .await;

        delay_for(Duration::from_secs_f64(0.5)).await;

        let mut client = Client::new("0.0.0.0:8123", &u2).await.unwrap();

        client.send(Message::String("Yeet".into())).await.unwrap();

        delay_for(Duration::from_secs_f64(0.5)).await;

        server.stop().await.unwrap();
    }
}
