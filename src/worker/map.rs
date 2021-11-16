use serde_json;
use std::collections::HashMap;
use std::fs::read_to_string;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn map(file: &str, output_file: &str) {
    let ip = read_to_string(file).unwrap();
    let mut hm: HashMap<&str, u32> = HashMap::new();
    for word in ip.split(&[' ', '.', ',', '\n', '\t', '?', '!', '\'', '\"', '_', '-'][..]) {
        if word.len() == 0 {
            continue;
        }
        let ctr = hm.entry(word).or_insert(0);
        *ctr += 1;
    }
    let op = serde_json::to_string(&hm).unwrap();
    let mut file = File::create(output_file).await.unwrap();
    file.write_all(op.as_bytes()).await.unwrap();
}
