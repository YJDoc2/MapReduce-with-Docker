use crate::ip_finder::{get_ip_list, get_self_ip};
use manager::MasterMessage;
use manager::{spwan_manager, Tasks};
use std::net::Ipv4Addr;
use std::str::FromStr;

use tokio::sync::mpsc;

const MAP_SPLIT: u8 = 5;

fn get_splitfile_name(idx: u8) -> String {
    format!("map_split_{}.txt", idx)
}

async fn spawn_tracker(manager: mpsc::Sender<Tasks>, rcvr: mpsc::Receiver<()>) {
    let mut rcvr = rcvr;

    for i in 1..=MAP_SPLIT {
        let name = get_splitfile_name(i);
        let msg = MasterMessage::MapDirective { input_file: name };
        if let Err(_) = manager.clone().send(Tasks::Allocate { message: msg }).await {
            println!("Error in sending map file information to nodes");
        } else {
            println!("sent {} task", i);
        }
    }
    if let Some(()) = rcvr.recv().await {
        println!("Done!");
    }
}

pub async fn master_main() -> Result<(), Box<dyn std::error::Error>> {
    let self_ip = Ipv4Addr::from_str("127.0.0.1").unwrap(); //get_self_ip();
    let connected_ips = vec!["127.0.0.1"]; //get_ip_list(&self_ip).await;

    let (sender, rcvr) = mpsc::channel::<()>(5);
    let manager = spwan_manager(self_ip, sender).await?;

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

    spawn_tracker(manager.clone(), rcvr).await;
    Ok(())
}
