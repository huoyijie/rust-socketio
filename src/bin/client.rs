use bytes::Bytes;
use rust_socketio::SocketIO;
use std::io;
use tokio::time;
use tokio::time::Duration;

const SECRET_KEY: &[u8; 32] = b"ade2ff15798d44959d2846974bbf0bb3";
const SECRET_IV: &[u8; 16] = b"bd3c01bfb8c2edca";

#[tokio::main]
async fn main() -> io::Result<()> {
  let io: SocketIO = SocketIO::new("0.0.0.0:0", *SECRET_KEY, *SECRET_IV).await?;
  let mut socket = io.connect("127.0.0.1:8080").await?;
  let mut interval = time::interval(Duration::from_secs(3));
  loop {
    tokio::select! {
      res = socket.read() => {
        if let Some(bytes) = res? {
          eprintln!("received {:?}", bytes);
        } else {
          println!("EOF");
          io.shutdown().await?;
          break;
        }
      }
      _ = interval.tick() => {
        socket.write(Bytes::copy_from_slice(&[1u8; 2048])).await?;
      }
      _ = tokio::signal::ctrl_c() => {
        println!("ctrl-c received!");
        io.shutdown().await?;
        break;
      }
    }
  }
  Ok(())
}
