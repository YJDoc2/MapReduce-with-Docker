use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn extract_num(f: &str) -> u8 {
    f.rsplit_once('_')
        .unwrap()
        .1
        .rsplit_once('.')
        .unwrap()
        .0
        .parse()
        .unwrap()
}

fn format_reduced<'a>(reduced: &'a HashMap<&str, u32>) -> Vec<u8> {
    let mut joined: String = "".to_owned();
    for (k, v) in reduced.iter() {
        joined = format!("{}\n{} {}", joined, k, v);
    }
    joined = format!("{}\n", joined);
    return joined.as_bytes().to_vec();
}

pub async fn reduce(file: &str) {
    let mut f = File::open(file).await.unwrap();
    let mut contents = vec![];
    f.read_to_end(&mut contents).await.unwrap();
    let ip = String::from_utf8(contents).unwrap();
    let mut hm: HashMap<&str, u32> = HashMap::new();
    for line in ip.lines() {
        if line.trim().len() == 0 {
            continue;
        }
        let (k, v) = line.trim().split_once(' ').unwrap();
        let ctr = hm.entry(k).or_insert(0);
        *ctr += v.parse::<u32>().unwrap();
    }
    let idx = extract_num(file);
    let op = format_reduced(&hm);
    let mut file = File::create(format!("reduce_split_{}.txt", idx))
        .await
        .unwrap();
    file.write_all(&op).await.unwrap();
}
