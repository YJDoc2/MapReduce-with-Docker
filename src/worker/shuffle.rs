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

pub async fn shuffle(name: &str, file: &str, splits: usize) {
    let mut f = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(file)
        .await
        .unwrap();
    let mut contents = vec![];
    f.read_to_end(&mut contents).await.unwrap();
    // each shuffle split has to take care that its's own file is truncated
    // after reading from it is done
    f.set_len(0).await.unwrap();
    // close the file, so that if any other shuffle worker wants
    // to write to it, it can do so
    drop(f);
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
    // This is a bit hacky, but should work fine
    // as long as the splits done are of equal size, and the input size is big enough
    // this basically waits 100ms for rest of the shuffle to read from their files, in case they haven't yet
    // as we overwrite the original files for shuffle results.
    // If the splits are equal sized, then there is a low chance that
    // when one node has finished all of its read and processing some other node is still reading from a file
    let wd = Duration::from_millis(100);
    sleep(wd).await;

    for (k, v) in shuffled.iter() {
        let wait_time: u64 = rng.gen_range(10..100);
        let wd = Duration::from_millis(wait_time);
        let op = format_shuffled(v);
        // wait for random duration so that we don't mistakenly overwrite when
        // some other worker is writing
        sleep(wd).await;
        // this is sync write, as thread getting suspended during write
        // can cause problems
        let mut file = OpenOptions::new()
            .append(true)
            .open(fpath.join(format!("{}_split_{}.txt", name, k)))
            .unwrap();
        file.write_all(&op).unwrap();
    }
}
