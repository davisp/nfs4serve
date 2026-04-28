use std::net::{SocketAddr, ToSocketAddrs as _};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result, anyhow};
use tokio::net::TcpListener;

use crate::tcp::TcpConnection;

#[derive(Debug)]
pub struct NFSv41ServerInner {
    address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct NFSv41Server {
    inner: Arc<Mutex<NFSv41ServerInner>>,
}

impl NFSv41Server {
    pub async fn new(addr: &str) -> Result<Self> {
        let Some(address) = addr
            .to_socket_addrs()
            .context("Error parsing or resolving server listen address.")?
            .nth(0)
        else {
            return Err(anyhow!(
                "No addresses found for the provided server listen address."
            ));
        };

        let inner = NFSv41ServerInner { address };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    /// Main loop
    ///
    /// # Panics
    ///
    /// If the mutex is poisoned.
    pub async fn serve(&self) -> Result<()> {
        let address = {
            let guard = self.inner.lock().expect("Server lock was poisoned.");
            guard.address
        };

        let listener = TcpListener::bind(address)
            .await
            .context("Error binding server listener socket.")?;

        log::info!("Server started. Waiting for connections.");
        loop {
            let (socket, addr) = listener
                .accept()
                .await
                .context("Error accepting next client connection.")?;

            let conn = TcpConnection::new(socket, addr)
                .context("Error creating client connection.")?;

            tokio::spawn(async move {
                match conn.handle().await {
                    Ok(()) => (),
                    Err(err) => {
                        log::info!("Client disconnected with error: {err:#?}");
                    }
                }
            });
        }
    }
}
