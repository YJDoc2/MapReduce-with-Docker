use crate::messages::MasterMessage;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::Ipv4Addr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub const SOCKET_ADDR: u16 = 7000;

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub enum WorkerType {
    None,
    Mapper,
    Shuffler,
    Reducer,
}

#[derive(Hash, PartialEq, Eq)]
pub struct Worker {
    assigned_type: WorkerType,
    ip_addr: Ipv4Addr,
}

pub struct Manager {
    free_pool: VecDeque<Worker>,
    assigned_set: HashMap<String, Worker>,
    pub pending: VecDeque<MasterMessage>,
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            free_pool: VecDeque::new(),
            assigned_set: HashMap::new(),
            pending: VecDeque::new(),
        }
    }

    pub fn add_worker(&mut self, addr: Ipv4Addr) {
        self.free_pool.push_back(Worker {
            assigned_type: WorkerType::None,
            ip_addr: addr,
        })
    }

    pub async fn allocate(&mut self, message: MasterMessage) {
        if self.free_pool.len() == 0 {
            self.pending.push_back(message);
            return;
        }
        let worker = self.free_pool.pop_front().unwrap();
        self.send_message(worker, &message).await;
    }

    pub async fn send_message(&mut self, mut worker: Worker, message: &MasterMessage) {
        let wtype = match &message {
            MasterMessage::MapDirective { input_file: _ } => WorkerType::Mapper,
            MasterMessage::ReduceDirective { input_file: _ } => WorkerType::Reducer,
            MasterMessage::ShuffleDirective {
                input_file: _,
                splits: _,
            } => WorkerType::Shuffler,
        };
        println!("{} {}", worker.ip_addr, SOCKET_ADDR);
        let mut stream = TcpStream::connect((worker.ip_addr, SOCKET_ADDR))
            .await
            .unwrap();
        stream.write_all(&message.to_u8_vec()).await.unwrap();
        worker.assigned_type = wtype;
        self.assigned_set
            .insert(String::from(worker.ip_addr.to_string()), worker);
    }

    pub async fn worker_done(&mut self, ip: &str) {
        if let Some(x) = self.assigned_set.remove(ip) {
            if let Some(m) = self.pending.pop_front() {
                self.send_message(x, &m).await;
                return;
            }
            self.free_pool.push_back(x);
        }
    }

    pub fn tasks_done(&self) -> bool {
        self.pending.len() == 0 && self.assigned_set.len() == 0
    }
}
