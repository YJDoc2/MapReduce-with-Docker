use serde_json;
use std::collections::HashMap;
use std::fs::read_to_string;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

fn wordcount(ip: &str) -> String {
    let mut hm: HashMap<&str, u32> = HashMap::new();
    for word in ip.split(&[' ', '.', ',', '\n', '\t', '?', '!', '\'', '\"', '_', '-'][..]) {
        if word.len() == 0 {
            continue;
        }
        let ctr = hm.entry(word).or_insert(0);
        *ctr += 1;
    }
    return serde_json::to_string(&hm).unwrap();
}

fn matrix(ip: &str) -> String {
    unimplemented!();
}

pub async fn map(job_name: &str, file: &str) {
    let ip = read_to_string(file).unwrap();
    let op = match job_name {
        "wordcount" => wordcount(&ip),
        "matrix" => matrix(&ip),
        _ => panic!("Unknown Job encountered"),
    };
    let mut file = File::create(file).await.unwrap();
    file.write_all(op.as_bytes()).await.unwrap();
}
