use super::ip_finder::{get_ip_list, get_self_ip};
use super::manger_controller::{spwan_manager, Tasks};
use manager::{MasterMessage, SlaveMessage};
use serde_json;
use std::net::Ipv4Addr;
use std::str::FromStr;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

const SOCKET_ADDR: u16 = 7000;

const MAP_SPLIT: u8 = 5;

fn get_splitfile_name(idx: u8) -> String {
    format!("map_split_{}.txt", idx)
}

pub async fn master_main() -> Result<(), Box<dyn std::error::Error>> {
    let self_ip = Ipv4Addr::from_str("127.0.0.1").unwrap(); //get_self_ip();
    let connected_ips = vec!["127.0.0.1"]; //get_ip_list(&self_ip).await;
    let manager = spwan_manager();
    for ip in connected_ips {
        if let Err(_) = manager
            .send(Tasks::AddWorker {
                ip: Ipv4Addr::from_str(&ip).unwrap(),
            })
            .await
        {
            println!("Error in adding worker to manager");
        }
    }

    // todo split the file here

    for i in 1..=MAP_SPLIT {
        let name = get_splitfile_name(i);
        let msg = MasterMessage::MapDirective { input_file: name };
        if let Err(_) = manager.clone().send(Tasks::Allocate { message: msg }).await {
            println!("Error in sending map file information to nodes");
        };
    }

    let listener = TcpListener::bind((self_ip, 8000 as u16)).await?;
    loop {
        let (mut socket, addr) = listener.accept().await?;
        let addrv4 = match addr {
            std::net::SocketAddr::V6(_) => panic!("Needs to support ip v6"),
            std::net::SocketAddr::V4(a) => a.ip().clone(),
        };
        let m_clone = manager.clone();
        tokio::spawn(async move {
            let mut buf = [0; 50];
            loop {
                match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(_) => {
                        let msg = String::from_utf8(Vec::from(&buf[0..6])).unwrap();
                        let msg: SlaveMessage = serde_json::from_str(&msg).unwrap();
                        match msg {
                            SlaveMessage::Done => {
                                if let Err(_) = m_clone.send(Tasks::FreeWorker { ip: addrv4 }).await
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
}
