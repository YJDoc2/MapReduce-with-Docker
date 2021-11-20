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
    let mut hm: HashMap<(usize, usize), Vec<(&str, usize, f32)>> = HashMap::new();
    for line in ip.lines() {
        let splitted: Vec<_> = line.split_ascii_whitespace().collect();
        let i = splitted[0].parse().unwrap();
        let k = splitted[1].parse().unwrap();
        let name = splitted[2];
        let j = splitted[3].parse().unwrap();
        let val = splitted[4].parse().unwrap();
        let entry = hm.entry((i, k)).or_default();
        entry.push((name, j, val));
    }
    let mut ret: HashMap<(usize, usize), f32> = HashMap::with_capacity(500);
    for ((i, k), v) in hm.into_iter() {
        let mut temp_a = HashMap::new();
        let mut temp_b = HashMap::new();
        for (name, j, v) in v.into_iter() {
            if name == "A" {
                temp_a.insert(j, v);
            } else {
                temp_b.insert(j, v);
            }
        }
        let mut res = 0.0;
        for j in 0..50 {
            res += temp_a.get(&j).unwrap_or(&0.0) * temp_b.get(&j).unwrap_or(&0.0);
        }
        ret.insert((i, k), res);
    }
    let mut s = String::new();
    for ((i, j), v) in ret.into_iter() {
        s.push_str(&format!("{} {} {}\n", i, j, v));
    }

    return s.as_bytes().to_vec();
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
