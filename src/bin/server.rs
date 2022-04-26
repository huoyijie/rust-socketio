use rust_socketio::SocketIO;
use std::io;

const SECRET_KEY: &[u8; 32] = b"ade2ff15798d44959d2846974bbf0bb3";
const SECRET_IV: &[u8; 16] = b"bd3c01bfb8c2edca";

#[tokio::main]
async fn main() -> io::Result<()> {
  let mut io = SocketIO::new("0.0.0.0:8080", *SECRET_KEY, *SECRET_IV).await?;
  loop {
    tokio::select! {
      res = io.accept() => {
        let mut socket = res.unwrap();
        tokio::spawn(async move {
          loop {
            match socket.read().await {
              Ok(Some(bytes)) => {
                if let Err(e) = socket.write(bytes).await {
                  eprintln!("socket.write failed; err = {:?}", e);
                }
              }
              Ok(None) => return,
              Err(e) => {
                eprintln!("socket.read failed; err = {:?}", e);
                return;
              }
            };
          }
        });
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
