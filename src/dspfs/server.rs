use crate::global_store::{SharedStore, Store};
use crate::message::{ErrorMessage, Message};
use crate::stream::EncryptedStream;
use crate::user::PrivateUser;
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::ops::Deref;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct Server<S: Store + 'static> {
    listener: TcpListener,
    store: SharedStore<S>,
    pub addr: SocketAddr,
}

pub struct ServerHandle {
    pub(self) stop_channel: Sender<()>,

    pub addr: SocketAddr,
}

impl ServerHandle {
    pub async fn stop(mut self) -> Result<()> {
        self.stop_channel
            .send(())
            .await
            .context("failed to stop server")?;
        Ok(())
    }
}

impl<S: Store + 'static> Server<S> {
    // Creates a server struct with the tcplistener
    pub async fn new(addr: impl ToSocketAddrs, store: SharedStore<S>) -> Result<Self> {
        let sock_addr = addr
            .to_socket_addrs()
            .await?
            .next()
            .context("couldn't get socket address")?;

        Ok(Server {
            addr: sock_addr,
            listener: TcpListener::bind(&addr).await?,
            store: store.clone(),
        })
    }

    // Starts listening for requests
    // contains a loop checking for errors
    pub async fn start(mut self) -> ServerHandle {
        let (tx, mut rx) = channel(2);

        let addr = self.addr;

        log::info!("Starting server");
        // Outer loop for catching errors
        tokio::spawn(async move {
            while let Err(e) = self.internal_start(&mut rx).await {
                log::error!("an error occurred; error = {:?}", e);
            }
        });

        ServerHandle {
            stop_channel: tx,
            addr,
        }
    }

    // Inner loop for receiving messages and calling on [process]
    async fn internal_start(&mut self, stopper: &mut Receiver<()>) -> Result<()> {
        log::info!("Now accepting requests");

        loop {
            select! {
                _ = stopper.recv() => {
                    // If we receive stop signal stop
                    return Ok(())
                }
                accepted = self.listener.accept() => {
                    // Normal message
                    let (stream, addr) = accepted.context("failed to accept connection")?;
                    let local_store = self.store.clone();

                    // process the message
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(local_store, stream, addr).await {
                            log::error!("an error occurred; error = {:?}", e);
                        }
                    });
                }
            }
        }
    }
}

// Actually process the incoming requests
async fn handle_connection<S: Store>(
    store: SharedStore<S>,
    stream: impl AsyncReadExt + AsyncWriteExt + Unpin + Send + Sync,
    _addr: SocketAddr,
) -> Result<()> {
    let guard = store.read().await;
    // FIXME
    let user = PrivateUser::load_from_store(guard.deref().deref())
        .context("Couldn't load user from global_store")?;
    let mut es = EncryptedStream::receiver(stream, user)
        .await
        .context("Couldn't establish secure connection")?;

    // Check type of message
    // FIXME: Change limit
    while let Ok(message) = es.recv_message(4096).await {
        log::info!("{:?}", message);
        match message {
            Message::Init { .. } => {
                // drop connection
                return Err(anyhow::anyhow!("Connection reinitialized by client"));
            }
            Message::String(s) => {
                log::info!("{}", s);
            }
            Message::FileBlockRequest {
                groupuuid,
                filehash,
                index,
            } => {
                // File Request:
                // get group
                let group =
                    store.read().await.get_group(groupuuid)?.ok_or_else(|| {
                        anyhow::anyhow!("Group with uuid {} not found", groupuuid)
                    })?;

                // verify user is actually in that group
                if !group.users.contains(&es.other_user) {
                    return Err(anyhow::anyhow!("Client not in group"));
                }

                let group = group.reload(store.clone())?;

                // send file
                let block = if let Some(s) = group.get_block_contents(filehash, index).await? {
                    s
                } else {
                    es.send_message(Message::Error(ErrorMessage::FileNotFound))
                        .await?;
                    es.close();
                    return Ok(());
                };

                es.send_message(Message::FileBlock(block)).await?;
            }
            message => log::error!("Received invalid message: {:?}", message),
        }
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use crate::dspfs::client::Client;
    use crate::dspfs::server::{handle_connection, Server};
    use crate::fs::file::File;
    use crate::fs::group::StoredGroup;
    use crate::global_store::inmemory::InMemoryStore;
    use crate::global_store::Store;
    use crate::init;
    use crate::message::Message;
    use crate::stream::EncryptedStream;
    use crate::user::PrivateUser;
    use std::io::Write;
    use std::ops::Deref;
    use tempfile::tempdir;
    use tokio::time::{delay_for, Duration};

    #[tokio::test]
    pub async fn test_simple_stream() {
        init();

        let (u1, doc1) = PrivateUser::new("Test1").unwrap();
        let (u2, _) = PrivateUser::new("Test2").unwrap();

        let store1 = InMemoryStore::default().shared();

        store1.write().await.set_signing_key(doc1).unwrap();
        store1.write().await.set_self_user(u1.to_owned()).unwrap();

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

    #[tokio::test]
    pub async fn test_block_request() {
        // Create Store
        let store1 = InMemoryStore::test_store("test1").unwrap();

        let tmpdir = tempdir().unwrap();
        let mut path = tmpdir.path().to_path_buf();

        // Initiate encrypted stream
        let us = PrivateUser::load_from_store(store1.read().await.deref().deref()).unwrap();

        // Create group
        let mut group = StoredGroup::new(path.clone());
        group.users.push(us.public_user().clone());
        let guuid = group.uuid;

        std::fs::create_dir_all(group.dspfs_folder()).unwrap();

        path.push("test");
        let path = path.to_string_lossy().into_owned();

        let mut test_file = std::fs::File::create(&path).unwrap();

        test_file.write_all(b"Hello World!\n").unwrap();

        store1.write().await.add_group(group).unwrap();
        let mut loaded_group = store1
            .read()
            .await
            .get_group(guuid)
            .unwrap()
            .unwrap()
            .reload(store1.clone())
            .unwrap();

        // Create file with the hash we will ask
        let file = File::new_empty("test".into());
        let fhash = file.hash.clone();
        loaded_group
            .add_file(&us.public_user(), file)
            .await
            .unwrap();

        let (tx, rx) = tokio::net::UnixStream::pair().unwrap();
        let es = EncryptedStream::initiator(tx, &us);

        tokio::spawn(async move {
            handle_connection(store1.clone(), rx, "127.0.0.1:8000".parse().unwrap())
                .await
                .unwrap();
        });

        let mut es = es.await.unwrap();

        es.send_message(Message::FileBlockRequest {
            groupuuid: guuid,
            filehash: fhash,
            index: 0,
        })
        .await
        .unwrap();

        let msg = es.recv_message(1024).await.unwrap();

        if let Message::FileBlock(bytes) = msg {
            assert_eq!(bytes, b"Hello World!\n")
        } else {
            panic!();
        }
    }
}
