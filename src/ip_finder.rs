use async_std::task;
use std::net::Ipv4Addr;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::Duration;

pub fn get_self_ip() -> Ipv4Addr {
    let awk = Command::new("awk")
        .arg("END{print $1}")
        .arg("/etc/hosts")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to obtain own ip address using awk");
    let out = awk.wait_with_output().expect("Initial awk failed").stdout;
    let ip = String::from_utf8(out).unwrap();
    Ipv4Addr::from_str(ip.trim()).unwrap()
}

pub async fn get_ip_list(own_ip: &Ipv4Addr) -> Vec<String> {
    let ip_string = own_ip.to_string();
    let temp = ip_string.trim().rsplit_once(".").unwrap().0;
    let nmap = Command::new("nmap")
        .arg("-n")
        .arg("-sn")
        .arg(format!("{}.*", temp))
        .arg("-oG")
        .arg("-")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run nmap");

    let nmap_out = nmap.stdout.expect("Failed to Run nmap");

    let awk = Command::new("awk")
        .arg("/Up/{print $2}")
        .stdin(Stdio::from(nmap_out))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run awk process");

    task::sleep(Duration::from_secs(2)).await;

    let output = awk.wait_with_output().expect("Failed to wait on awk");
    let list = String::from_utf8(output.stdout).unwrap();
    list.lines()
        .filter(|s| **s != ip_string)
        .map(|s| s.to_owned())
        .collect::<Vec<_>>()
}
