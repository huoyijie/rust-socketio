use crate::{socket::Builder, Socket};
use rust_udpack::Udpack;
use std::io;
use tokio::net::ToSocketAddrs;

/// Adds a layer of abstraction over Udpack to provide secure and frame-based data transfer.
#[derive(Debug)]
pub struct SocketIO {
  /// udpack instance
  udpack: Udpack,

  /// secret key for aes-256-cbc
  secret_key: [u8; 32],

  /// secret iv for aes-256-cbc
  secret_iv: [u8; 16],
}

impl SocketIO {
  /// construct a instance of SocketIO with the bind address, secret key and secret iv.
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

  /// accept a socket instance that constructed from a transport.
  pub async fn accept(&mut self) -> Option<Socket> {
    if let Some(transport) = self.udpack.accept().await {
      return Some(Builder::new(transport, self.secret_key, self.secret_iv));
    } else {
      return None;
    }
  }

  /// connect to the dest address and return the socket instance or an error.
  pub async fn connect(&self, dst_addr: &str) -> io::Result<Socket> {
    let transport = self.udpack.connect(dst_addr).await?;
    Ok(Builder::new(transport, self.secret_key, self.secret_iv))
  }

  /// shutdown all sockets.
  pub async fn shutdown(self) -> io::Result<()> {
    self.udpack.shutdown().await
  }
}
