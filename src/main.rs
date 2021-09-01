mod ip_finder;
mod mapreduce;
mod master;
mod slave;
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
            if s == "slave" {
                return slave::slave_main().await.unwrap();
            }
            eprintln!("Error : TYPE must be master or slave, got {}", s)
        }
        Err(_) => {}
    };
}
