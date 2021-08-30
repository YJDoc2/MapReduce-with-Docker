use crate::ip_finder::{get_ip_list, get_self_ip};
use manager::MasterMessage;
use manager::{spwan_manager, Tasks};
use std::net::Ipv4Addr;
use std::str::FromStr;

use tokio::sync::mpsc;

const MAP_SPLIT: u8 = 5;

fn get_splitfile_name(task: &str, idx: u8) -> String {
    format!("{}_split_{}.txt", task, idx)
}

// should be changed later?
fn get_input_file() -> String {
    std::env::var("INPUT").unwrap()
}

// This is still hacky, but works for now
fn split_input_file() {
    use std::io::Write;
    let file = get_input_file();
    let base = file.rsplit_once("/").unwrap().0;
    let ip = std::fs::read_to_string(file.clone()).unwrap();
    let lines: Vec<_> = ip.split('\n').collect();
    let d = (lines.len() as f32 / MAP_SPLIT as f32).ceil() as usize;
    let mut start = 0;
    let mut end;
    for i in 1..=MAP_SPLIT {
        end = if start + d <= lines.len() {
            start + d
        } else {
            lines.len()
        };
        let mut opts = std::fs::OpenOptions::new();
        let _fname = format!("{}/{}", base, get_splitfile_name("map", i));
        println!("{}", _fname);
        let mut _f = opts.write(true).create_new(true).open(_fname).unwrap();
        for line in &lines[start..end] {
            _f.write(line.as_bytes()).unwrap();
            _f.write(&['\n' as u8]).unwrap();
        }
        start = end;
    }
}

async fn spawn_tracker(manager: mpsc::Sender<Tasks>, rcvr: mpsc::Receiver<()>) {
    let mut rcvr = rcvr;

    for i in 1..=MAP_SPLIT {
        let name = get_splitfile_name("map", i);
        let msg = MasterMessage::MapDirective { input_file: name };
        if let Err(_) = manager.clone().send(Tasks::Allocate { message: msg }).await {
            println!("Error in sending map file information to nodes");
        } else {
            println!("queued map task {}", i);
        }
    }
    if let Some(()) = rcvr.recv().await {
        println!("All map done");
    }

    for i in 1..=MAP_SPLIT {
        let name = get_splitfile_name("map", i);
        let msg = MasterMessage::ShuffleDirective {
            input_file: name,
            splits: MAP_SPLIT,
        };
        if let Err(_) = manager.clone().send(Tasks::Allocate { message: msg }).await {
            println!("Error in sending shuffle file information to nodes");
        } else {
            println!("queued shuffle task {}", i);
        }
    }

    if let Some(()) = rcvr.recv().await {
        println!("All shuffle done");
    }

    for i in 1..=MAP_SPLIT {
        let name = get_splitfile_name("shuffle", i);
        let msg = MasterMessage::ReduceDirective { input_file: name };
        if let Err(_) = manager.clone().send(Tasks::Allocate { message: msg }).await {
            println!("Error in sending reduced file information to nodes");
        } else {
            println!("queued reduce task {}", i);
        }
    }

    if let Some(()) = rcvr.recv().await {
        println!("All reduce done");
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

    split_input_file();

    spawn_tracker(manager.clone(), rcvr).await;
    Ok(())
}
