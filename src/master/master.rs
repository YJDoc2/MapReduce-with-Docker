use crate::ip_finder::{get_ip_list, get_self_ip};
use manager::MasterMessage;
use manager::{spwan_manager, Tasks};
use std::path::PathBuf;
use tokio::sync::mpsc;

// should be changed later?
fn get_input_file() -> String {
    std::env::var("INPUT").unwrap()
}

pub async fn master_main() -> Result<(), Box<dyn std::error::Error>> {
    let self_ip = get_self_ip();
    let connected_ips = get_ip_list(&self_ip).await;
    Ok(())
}
