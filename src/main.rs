mod mapreduce;
mod master;

#[tokio::main]
async fn main() {
    master::master_main().await.unwrap();
}
