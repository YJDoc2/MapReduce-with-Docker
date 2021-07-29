use manager::Manager;
use manager::MasterMessage;
use std::net::Ipv4Addr;
use tokio::sync::mpsc;

pub enum Tasks {
    AddWorker { ip: Ipv4Addr },
    Allocate { message: MasterMessage },
    FreeWorker { ip: Ipv4Addr },
}

pub fn spwan_manager() -> mpsc::Sender<Tasks> {
    let (tx, mut rx) = mpsc::channel(100);
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
                }
            }
        }
    });
    return tx;
}
