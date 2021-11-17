use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn get_input_file() -> String {
    std::env::var("INPUT").unwrap()
}

#[allow(dead_code)]
fn generate_rand_matrix(count: usize, threshold: f32, range: f32, mr_name: &str, sn_name: &str) {
    use rand::prelude::*;
    let mut a: Vec<Vec<f32>> = Vec::with_capacity(count);
    let mut b: Vec<Vec<f32>> = Vec::with_capacity(count);
    let mut rng = thread_rng();
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(mr_name)
        .unwrap();
    for i in 0..count {
        let mut temp = Vec::with_capacity(count);
        for j in 0..count {
            let decider: f32 = rng.gen();
            if decider < threshold {
                let rand_num: f32 = rng.gen_range(0.0..range);
                writeln!(f, "A {} {} {}", i, j, rand_num).unwrap();
                temp.push(rand_num);
            } else {
                temp.push(0.0);
            }
        }
        a.push(temp);
    }
    for i in 0..count {
        let mut temp = Vec::with_capacity(count);
        for j in 0..count {
            let decider: f32 = rng.gen();
            if decider < threshold {
                let rand_num: f32 = rng.gen_range(0.0..range);
                writeln!(f, "B {} {} {}", i, j, rand_num).unwrap();
                temp.push(rand_num);
            } else {
                temp.push(0.0);
            }
        }
        b.push(temp);
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(sn_name)
        .unwrap();
    writeln!(f, "{}", serde_json::to_string(&[a, b]).unwrap()).unwrap();
}

#[allow(dead_code)]
fn wordcount_main() {
    let mut fpath = PathBuf::from(get_input_file());
    println!("Starting...");
    let ip = std::fs::read_to_string(fpath.clone()).unwrap();
    let mut hm = HashMap::new();
    for word in ip.split(&[' ', '.', ',', '\n', '\t', '?', '!', '\'', '\"', '_', '-'][..]) {
        if word.len() == 0 {
            continue;
        }
        let ctr = hm.entry(word).or_insert(0);
        *ctr += 1;
    }
    fpath.pop();
    let mut t = String::new();
    t = hm.iter().fold(t, |t, (k, v)| t + &format!("{} {}\n", k, v));
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(fpath.join("singlenode.txt"))
        .unwrap();
    writeln!(f, "{}", t).unwrap();
}

#[allow(dead_code)]
fn matrix_main() {
    let mut fpath = PathBuf::from(get_input_file());
    println!("Starting...");
    let ip = std::fs::read_to_string(fpath.clone()).unwrap();
    let [a, b]: [Vec<Vec<f32>>; 2] = serde_json::from_str(&ip).unwrap();
    fpath.pop();
    let mut ret = Vec::with_capacity(a.len() * a.len());
    // we know that these both have equal size, as we have generated he data
    for i in 0..a.len() {
        println!("iter {}/500", i + 1);
        for j in 0..a.len() {
            let mut sum = 0.0;
            for k in 0..a.len() {
                sum += a[i][k] * b[k][j];
            }
            ret.push(sum);
        }
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(fpath.join("singlenode.txt"))
        .unwrap();
    let t: usize = a.len();
    for (i, v) in ret.iter().enumerate() {
        writeln!(f, "{} {} {}", i / t, i % t, v).unwrap();
    }
}

fn main() {
    let time = Instant::now();
    matrix_main();
    println!("{:?}", time.elapsed());
}
