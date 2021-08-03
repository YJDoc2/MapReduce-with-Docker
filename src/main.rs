mod ip_finder;
mod mapreduce;
mod master;
mod slave;

#[tokio::main]
async fn main() {
    match std::env::var("TYPE") {
        Ok(s) => {
            if s == "master" {
                return master::master_main().await.unwrap();
            }
            if s == "slave" {
                return slave::slave_main().await.unwrap();
            }
            eprintln!("Error : TYPE must be master or slave, got {}", s)
        }
        Err(_) => {}
    };
}
