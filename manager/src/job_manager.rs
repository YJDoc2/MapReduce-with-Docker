use crate::broker::Tasks;
use crate::messages::*;
use std::collections::{HashMap, VecDeque};
use std::io::BufReader;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::broker::spwan_manager;

pub enum Splits {
    Max,
    Count(usize),
}

pub struct Job {
    name: String,
    id: usize,
    input_file: String,
    splits: Splits,
    remaining_count: usize,
    pipeline: VecDeque<PipelineTask>,
}

#[derive(Copy, Clone)]
enum TaskType {
    Map,
    Shuffle,
    Reduce,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let o = match self {
            Self::Map => "map",
            Self::Shuffle => "shuffle",
            Self::Reduce => "reduce",
        };
        write!(f, "{}", o)
    }
}

struct QueuedTask {
    id: usize,
    splits: usize,
    input_file: String,
    output_file: String,
    task_type: TaskType,
}

pub struct PipelineTask {
    task_type: TaskType,
}

pub struct JobManager {
    id_ctr: usize,
    manager: Sender<Tasks>,
    rcvr: Receiver<usize>,
    jobs: HashMap<usize, Job>,
    connected: usize,
}

#[inline]
fn get_splitfile_name(job_name: &str, task: &str, idx: usize) -> String {
    format!("{}_{}_split_{}.txt", job_name, task, idx)
}

fn split_file(job_name: &str, input_file: &str, splits: usize) {
    use std::io::BufRead;
    use std::io::Write;
    let mut fpath = PathBuf::from(input_file);
    // read the file size
    let file_size = std::fs::metadata(fpath.clone())
        .expect("Error in reading input file size")
        .len();
    // calculate split size
    let split_size = (file_size as f32 / splits as f32) as u64;

    let mut opts = std::fs::OpenOptions::new();
    let mut ip_file = opts
        .write(false)
        .read(true)
        .create_new(false)
        .open(fpath.clone())
        .unwrap();
    let buffered_file = BufReader::new(ip_file);
    fpath.pop();
    let mut buffered_split = buffered_file.lines();
    for i in 1..=splits {
        let mut opts = std::fs::OpenOptions::new();
        let mut _f = opts
            .write(true)
            .create_new(true)
            .open(fpath.join(get_splitfile_name(job_name, "map", i)))
            .unwrap();
        loop {
            if _f.metadata().expect("Cannot read metadata").len() >= split_size {
                break;
            }
            match buffered_split.next() {
                Some(l) => writeln!(_f, "{}", l.unwrap()).expect("Error in writing splitfile"),
                None => return, // this will only be reached in the very last file, so we can return
            };
        }
    }
}

impl JobManager {
    pub async fn new(
        self_ip: Ipv4Addr,
        connected: &[&str],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use std::str::FromStr;
        let (sender, rcvr) = channel::<usize>(connected.len() - 1);
        let manager = spwan_manager(self_ip, sender).await?;
        for ip in &connected[1..] {
            if let Err(_) = manager
                .send(Tasks::AddWorker {
                    ip: Ipv4Addr::from_str(ip).unwrap(),
                })
                .await
            {
                println!("Error in adding worker to manager");
            }
        }
        Ok(Self {
            id_ctr: 1,
            manager: manager,
            rcvr: rcvr,
            jobs: HashMap::new(),
            connected: connected.len() - 1,
        })
    }

    pub fn queue_job(&mut self, job: Job) {
        self.jobs.insert(self.id_ctr, job);
        self.id_ctr += 1;
    }

    pub async fn start(&mut self) {
        for (_, job) in &self.jobs {
            let splits = match job.splits {
                Splits::Max => self.connected,
                Splits::Count(0) => self.connected,
                Splits::Count(n) => n,
            };
            split_file(&job.name, &job.input_file, splits);
        }
        self.spawn_tracker().await;
    }

    pub async fn spawn_tracker(&mut self) {
        let rcvr = &mut self.rcvr;
        let manager = &mut self.manager;
        // at any time, at max, in the queue a single job will have num_workers number of
        // either map shuffle or reduce jobs.
        // thus we have to take care of at max total jobs * number of workers jobs
        let mut queue = VecDeque::with_capacity(self.jobs.len() * self.connected);
        for (id, job) in &mut self.jobs {
            let splits = match job.splits {
                Splits::Max => self.connected,
                Splits::Count(0) => self.connected,
                Splits::Count(n) => n,
            };
            job.remaining_count = splits;
            for i in 1..=splits {
                queue.push_back(QueuedTask {
                    id: *id,
                    input_file: get_splitfile_name(&job.name, "map", splits),
                    output_file: get_splitfile_name(&job.name, "shuffle", splits),
                    splits: splits,
                    task_type: TaskType::Map,
                });
            }
        }
        loop {
            if queue.len() <= 0 {
                break;
            }
            while let Some(task) = queue.pop_front() {
                let msg: MasterMessage = match task.task_type {
                    TaskType::Map => MasterMessage::MapDirective {
                        id: task.id,
                        input_file: task.input_file,
                        output_file: task.output_file,
                    },
                    TaskType::Shuffle => MasterMessage::ShuffleDirective {
                        id: task.id,
                        input_file: task.input_file,
                        splits: self.connected,
                    },
                    TaskType::Reduce => MasterMessage::ReduceDirective {
                        id: task.id,
                        input_file: task.input_file,
                        output_file: task.output_file,
                    },
                };
                if let Err(_) = manager.clone().send(Tasks::Allocate { message: msg }).await {
                    println!("Error in sending information to nodes");
                } else {
                    println!("queued task");
                }
            }
            if let Some(id) = rcvr.recv().await {
                let job: &mut Job = self
                    .jobs
                    .get_mut(&id)
                    .expect("Invalid id received from workers");
                job.remaining_count -= 1;
                if job.remaining_count == 0 {
                    let new_task = match job.pipeline.pop_front() {
                        Some(task) => task.task_type,
                        None => continue, // Note that this will go to next iteration, so any code after if let...
                                          // at the start of this will be ignored in case None
                    };
                    let next_task = match job.pipeline.get(0) {
                        Some(task) => task.task_type.to_string(),
                        None => "result".to_owned(),
                    };
                    let splits = match job.splits {
                        Splits::Max => self.connected,
                        Splits::Count(0) => self.connected,
                        Splits::Count(n) => n,
                    };
                    job.remaining_count = splits;
                    for i in 1..=splits {
                        queue.push_back(QueuedTask {
                            id: job.id,
                            input_file: get_splitfile_name(
                                &job.name,
                                &new_task.to_string(),
                                splits,
                            ),
                            output_file: get_splitfile_name(&job.name, &next_task, splits),
                            splits: splits,
                            task_type: new_task,
                        });
                    }
                }
            }
        }
    }
}
