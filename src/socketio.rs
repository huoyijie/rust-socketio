use crate::{socket::Builder, Socket};
use rust_udpack::Udpack;
use std::io;
use tokio::net::ToSocketAddrs;

/// Adds a layer of abstraction over Udpack to provide secure and frame-based data transfer.
#[derive(Debug)]
pub struct SocketIO {
  udpack: Udpack,
  secret_key: [u8; 32],
  secret_iv: [u8; 16],
}

impl SocketIO {
  pub async fn new<A: ToSocketAddrs>(
    bind_addr: A,
    secret_key: [u8; 32],
    secret_iv: [u8; 16],
  ) -> io::Result<Self> {
    let udpack = Udpack::new(bind_addr).await?;
    Ok(Self {
      udpack,
      secret_key,
      secret_iv,
    })
  }

  pub async fn accept(&mut self) -> Option<Socket> {
    if let Some(transport) = self.udpack.accept().await {
      return Some(Builder::new(transport, self.secret_key, self.secret_iv));
    } else {
      return None;
    }
  }

  pub async fn connect(&self, dst_addr: &str) -> io::Result<Socket> {
    let transport = self.udpack.connect(dst_addr).await?;
    Ok(Builder::new(transport, self.secret_key, self.secret_iv))
  }

  pub async fn shutdown(self) -> io::Result<()> {
    self.udpack.shutdown().await
  }
}
