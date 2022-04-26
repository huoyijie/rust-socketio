use bytes::{Buf, BufMut, Bytes, BytesMut};
use openssl::symm::{decrypt, encrypt, Cipher};
use rust_udpack::Transport;
use std::io;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

/// Socket Builder
pub struct Builder;
impl Builder {
  pub fn new(transport: Transport, secret_key: [u8; 32], secret_iv: [u8; 16]) -> Socket {
    Socket::new(transport, secret_key, secret_iv)
  }
}

/// Adds a layer of abstraction over Transport to provide secure and frame-based data transfer.
pub struct Socket {
  /// transport instance
  transport: Transport,

  /// secret key for aes-256-cbc
  secret_key: [u8; 32],

  /// secret iv for aes-256-cbc
  secret_iv: [u8; 16],

  /// read buffer
  rd_buf: BytesMut,

  /// frame decoder
  decoder: LengthDelimitedCodec,

  /// write buffer
  wr_buf: BytesMut,

  /// frame encoder
  encoder: LengthDelimitedCodec,

  /// cipher (aes_256_cbc) for encrypt and decrypt
  cipher: Cipher,
}

impl Socket {
  /// uuid of the socket instance
  pub fn uuid(&self) -> u64 {
    self.transport.uuid()
  }

  /// Decode and decrypt bytes from the received frame,
  /// and will be blocked if there is no frame.
  pub async fn read(&mut self) -> io::Result<Option<Bytes>> {
    loop {
      if let Some(bytes_mut) = self.decoder.decode(&mut self.rd_buf)? {
        if let Ok(cipher_bytes) = decrypt(
          self.cipher,
          &self.secret_key,
          Some(&self.secret_iv),
          &bytes_mut,
        ) {
          return Ok(Some(Bytes::copy_from_slice(&cipher_bytes)));
        } else {
          return Err(io::Error::new(io::ErrorKind::InvalidData, "decrypt error."));
        }
      }
      if let Some(bytes) = self.transport.read().await {
        self.rd_buf.put_slice(&bytes);
      } else {
        if self.rd_buf.len() > 0 {
          return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("imcomplete data."),
          ));
        } else {
          return Ok(None);
        }
      }
    }
  }

  /// Whether the socket is writable or not.
  pub async fn writable(&self) -> io::Result<bool> {
    self.transport.writable().await
  }

  /// Write and encrypt bytes to one frame, and send the frame out.
  pub async fn write(&mut self, bytes: Bytes) -> io::Result<()> {
    if let Ok(cipher_bytes) = encrypt(self.cipher, &self.secret_key, Some(&self.secret_iv), &bytes)
    {
      self
        .encoder
        .encode(Bytes::copy_from_slice(&cipher_bytes), &mut self.wr_buf)?;
      self
        .transport
        .write(self.wr_buf.copy_to_bytes(self.wr_buf.len()))
        .await?;
      Ok(())
    } else {
      Err(io::Error::new(io::ErrorKind::InvalidData, "encrypt error."))
    }
  }

  /// send a ping and expected a pong
  pub fn ping(&self) -> io::Result<()> {
    self.transport.ping()
  }

  /// shutdown the socket
  pub fn shutdown(&self) -> io::Result<()> {
    self.transport.shutdown()
  }

  /// drain the socket
  pub fn close(&self) -> io::Result<()> {
    self.transport.close()
  }

  /// construct a socket instance
  fn new(transport: Transport, secret_key: [u8; 32], secret_iv: [u8; 16]) -> Self {
    Self {
      transport,
      secret_key,
      secret_iv,
      rd_buf: BytesMut::new(),
      decoder: LengthDelimitedCodec::new(),
      wr_buf: BytesMut::new(),
      encoder: LengthDelimitedCodec::new(),
      cipher: Cipher::aes_256_cbc(),
    }
  }
}
