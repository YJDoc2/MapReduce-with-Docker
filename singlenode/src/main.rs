use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn get_input_file() -> String {
    std::env::var("INPUT").unwrap()
}
fn main() {
    let time = Instant::now();
    let mut fpath = PathBuf::from(get_input_file());
    println!("Starting...");
    let ip = std::fs::read_to_string(fpath.clone()).unwrap();
    let mut hm = HashMap::new();
    for word in ip.split(&[' ', '.', ',', '\n', '\t', '?', '!'][..]) {
        let ctr = hm.entry(word).or_insert(0);
        *ctr += 1;
    }
    fpath.pop();
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(fpath.join("singlenode.txt"))
        .unwrap();
    for (k, v) in hm.iter() {
        writeln!(f, "{} {}", k, v).unwrap();
    }
    println!("{:?}", time.elapsed());
}
