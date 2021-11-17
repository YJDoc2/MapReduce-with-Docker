use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn format_reduced<'a>(reduced: &'a HashMap<&str, u32>) -> Vec<u8> {
    let mut joined: String = "".to_owned();
    for (k, v) in reduced.iter() {
        joined = format!("{}\n{} {}", joined, k, v);
    }
    joined = format!("{}\n", joined);
    return joined.as_bytes().to_vec();
}

fn wordcount(ip: &str) -> Vec<u8> {
    let mut hm: HashMap<&str, u32> = HashMap::new();
    for line in ip.lines() {
        if line.trim().len() == 0 {
            continue;
        }
        let (k, v) = line.trim().split_once(' ').unwrap();
        let ctr = hm.entry(k).or_insert(0);
        *ctr += v.parse::<u32>().unwrap();
    }
    return format_reduced(&hm);
}

fn matrix(ip: &str) -> Vec<u8> {
    unimplemented!();
}

pub async fn reduce(job_name: &str, file: &str) {
    let mut f = match File::open(file).await {
        Ok(r) => r,
        Err(_) => return,
    };
    let mut contents = vec![];
    f.read_to_end(&mut contents).await.unwrap();
    let ip = String::from_utf8(contents).unwrap();
    let mut fpath = PathBuf::from(file);
    fpath.pop();
    let op = match job_name {
        "wordcount" => wordcount(&ip),
        "matrix" => matrix(&ip),
        _ => panic!("Unknown Job encountered"),
    };
    let mut file = File::create(file).await.unwrap();
    file.write_all(&op).await.unwrap();
}
