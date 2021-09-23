use super::{map, reduce, shuffle};
use crate::ip_finder::get_self_ip;
use manager::{MasterMessage, SlaveMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub const SOCKET: u16 = 7000;
pub const MASTER_SOCKET: u16 = 8000;

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

async fn init_work(msg: &MasterMessage) {
    match msg {
        MasterMessage::MapDirective { input_file } => map(input_file).await,
        MasterMessage::ReduceDirective { input_file } => reduce(input_file).await,
        MasterMessage::ShuffleDirective { input_file, splits } => {
            shuffle(input_file, *splits).await
        }
    }
}

pub async fn slave_main() -> Result<(), Box<dyn std::error::Error>> {
    let self_ip = get_self_ip();
    let listener = TcpListener::bind((self_ip, SOCKET)).await?;

    loop {
        let (mut socket, addr) = listener.accept().await?;
        let addrv4 = match addr {
            std::net::SocketAddr::V6(_) => panic!("Needs to support ip v6"),
            std::net::SocketAddr::V4(a) => a.ip().clone(),
        };
        tokio::spawn(async move {
            let mut buf = [0; 512];

            // In a loop, read data from the socket and write the data back.
            loop {
                let _ = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => {}
                    Ok(_) => {
                        let msg = get_string(&buf);
                        let msg: MasterMessage = serde_json::from_str(&msg).unwrap();
                        init_work(&msg).await;
                        let t = serde_json::to_string(&SlaveMessage::Done).unwrap();
                        let mut s = TcpStream::connect((addrv4, MASTER_SOCKET)).await.unwrap();
                        s.write_all(&Vec::from(t)).await.unwrap();
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
