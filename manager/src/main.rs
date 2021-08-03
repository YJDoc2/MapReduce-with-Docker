mod manager;
mod messages;
use crate::manager::Manager;
use messages::{MasterMessage, SlaveMessage};
use std::net::Ipv4Addr;
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};
pub const ADDR: &str = "127.0.0.1";

// Duplicate definition, extract into common later
fn get_string(buf: &[u8]) -> String {
    let mut end = 0;
    for (i, v) in buf.iter().enumerate() {
        if *v == 0 {
            end = i;
            break;
        }
    }

    return String::from_utf8(Vec::from(&buf[0..end])).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7000").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0; 512];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(_) => {
                        let msg = get_string(&buf);
                        let msg: MasterMessage = serde_json::from_str(&msg).unwrap();
                        sleep(Duration::from_millis(1000)).await;
                        let t = serde_json::to_string(&SlaveMessage::Done).unwrap();
                        let mut s = TcpStream::connect("127.0.0.1:8000").await.unwrap();
                        s.write_all(&Vec::from(t)).await.unwrap();
                        println!("{:?}", msg);
                        return;
                    }
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
            }
        });
    }
}
