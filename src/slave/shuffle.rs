use super::get_hash;
use rand::prelude::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::time::{sleep, Duration};

fn format_shuffled<'a>(shuffled: &'a [(&str, u32)]) -> Vec<u8> {
    let mut joined: String = "".to_owned();
    for v in shuffled {
        joined = format!("{}\n{} {}", joined, v.0, v.1);
    }
    joined = format!("{}\n", joined);
    return joined.as_bytes().to_vec();
}

pub async fn shuffle(file: &str, splits: usize) {
    let mut f = tokio::fs::File::open(file).await.unwrap();

    let mut contents = vec![];
    f.read_to_end(&mut contents).await.unwrap();
    let ip = String::from_utf8(contents).unwrap();
    let hm: HashMap<String, u32> = serde_json::from_str(&ip).unwrap();
    let mut shuffled: HashMap<usize, Vec<(&str, u32)>> = HashMap::new();
    for (k, v) in hm.iter() {
        let h = get_hash(k, splits);
        let entries = shuffled.entry(h).or_default();
        entries.push((k, *v));
    }
    let mut rng: StdRng = SeedableRng::from_entropy();
    let mut fpath = PathBuf::from(file);
    fpath.pop();

    for (k, v) in shuffled.iter() {
        let wait_time: u64 = rng.gen_range(10..100);
        let wd = Duration::from_millis(wait_time);
        let op = format_shuffled(v);
        // wait for random duration so that we don't mistakenly overwrite when
        // some other slave is writing
        sleep(wd).await;
        // this is sync write, as thread getting suspended during write
        // can cause problems
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(fpath.join(format!("shuffle_split_{}.txt", k)))
            .unwrap();
        file.write_all(&op).unwrap();
    }
}
