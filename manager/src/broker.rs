use crate::manager::Manager;
use crate::messages::{MasterMessage, SlaveMessage};
use serde_json;
use std::net::Ipv4Addr;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

const TASK_QUEUE_LENGTH: usize = 50;
const BROKER_PORT: u16 = 8000;

#[derive(Debug)]
pub enum Tasks {
    AddWorker { ip: Ipv4Addr },
    Allocate { message: MasterMessage },
    FreeWorker { ip: Ipv4Addr },
}

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

pub async fn spawn_listener(self_ip: Ipv4Addr, manager: mpsc::Sender<Tasks>) {
    tokio::spawn(async move {
        let listener = TcpListener::bind((self_ip, BROKER_PORT)).await.unwrap();
        loop {
            let (mut socket, addr) = listener.accept().await.unwrap();
            let addrv4 = match addr {
                std::net::SocketAddr::V6(_) => panic!("Needs to support ip v6"),
                std::net::SocketAddr::V4(a) => a.ip().clone(),
            };
            let m_clone = manager.clone();
            tokio::spawn(async move {
                let mut buf = [0; 512];
                loop {
                    match socket.read(&mut buf).await {
                        // socket closed
                        Ok(n) if n == 0 => return,
                        Ok(_) => {
                            let msg = get_string(&buf);
                            let msg: SlaveMessage = serde_json::from_str(&msg).unwrap();
                            match msg {
                                SlaveMessage::Done => {
                                    if let Err(_) =
                                        m_clone.send(Tasks::FreeWorker { ip: addrv4 }).await
                                    {
                                        println!("Error in freeing worker");
                                    } else {
                                        println!("One task done");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };
                }
            });
        }
    });
}

pub async fn spwan_manager(
    self_ip: Ipv4Addr,
    sender: mpsc::Sender<()>,
) -> Result<mpsc::Sender<Tasks>, Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(TASK_QUEUE_LENGTH);
    spawn_listener(self_ip, tx.clone()).await;
    tokio::spawn(async move {
        let mut manager = Manager::new();
        while let Some(task) = rx.recv().await {
            match task {
                Tasks::AddWorker { ip } => {
                    manager.add_worker(ip);
                }
                Tasks::Allocate { message } => {
                    manager.allocate(message).await;
                }
                Tasks::FreeWorker { ip } => {
                    manager.worker_done(&ip.to_string()).await;
                    if manager.tasks_done() {
                        sender.send(()).await.unwrap();
                    }
                }
            }
        }
    });
    return Ok(tx);
}
