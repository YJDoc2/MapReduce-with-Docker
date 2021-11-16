mod ip_finder;
mod mapreduce;
mod master;
mod worker;
use std::env;
use std::time::Instant;

#[tokio::main]
async fn main() {
    match env::var("TYPE") {
        Ok(s) => {
            if s == "master" {
                let time = Instant::now();
                master::master_main().await.unwrap();
                println!("{:?}", time.elapsed());
                return;
            }
            if s == "worker" {
                return worker::worker_main().await.unwrap();
            }
            eprintln!("Error : TYPE must be master or worker, got {}", s)
        }
        Err(e) => {
            eprintln!("Error {}", e)
        }
    };
}
