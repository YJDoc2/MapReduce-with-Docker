use crate::ip_finder::{get_ip_list, get_self_ip};
use manager::MasterMessage;
use manager::{spwan_manager, Tasks};
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::PathBuf;
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
    let mut fpath = PathBuf::from(get_input_file());
    let ip = std::fs::read_to_string(fpath.clone()).unwrap();
    let lines: Vec<_> = ip.split('\n').collect();
    let d = (lines.len() as f32 / MAP_SPLIT as f32).ceil() as usize;
    let mut start = 0;
    let mut end;
    fpath.pop();
    for i in 1..=MAP_SPLIT {
        end = if start + d <= lines.len() {
            start + d
        } else {
            lines.len()
        };
        let mut opts = std::fs::OpenOptions::new();
        let mut _f = opts
            .write(true)
            .create_new(true)
            .open(fpath.join(get_splitfile_name("map", i)))
            .unwrap();
        for line in &lines[start..end] {
            _f.write(line.as_bytes()).unwrap();
            _f.write(&['\n' as u8]).unwrap();
        }
        start = end;
    }
}

async fn spawn_tracker(manager: mpsc::Sender<Tasks>, rcvr: mpsc::Receiver<()>) {
    let mut rcvr = rcvr;
    let mut fpath = PathBuf::from(get_input_file());
    fpath.pop();
    for i in 1..=MAP_SPLIT {
        let name = fpath
            .join(get_splitfile_name("map", i))
            .to_str()
            .unwrap()
            .to_owned();
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
        let name = fpath
            .join(get_splitfile_name("map", i))
            .to_str()
            .unwrap()
            .to_owned();
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
        let name = fpath
            .join(get_splitfile_name("shuffle", i))
            .to_str()
            .unwrap()
            .to_owned();
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
    let self_ip = get_self_ip();
    let connected_ips = get_ip_list(&self_ip).await;
    println!("{:?}", connected_ips);
    let (sender, rcvr) = mpsc::channel::<()>(5);
    let manager = spwan_manager(self_ip, sender).await?;

    for ip in &connected_ips[1..] {
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
