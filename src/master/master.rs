use crate::ip_finder::{get_ip_list, get_self_ip};
use manager::{Job, JobManager, PipelineTask, TaskType};
use std::collections::VecDeque;

// should be changed later?
fn get_input_file() -> String {
    std::env::var("INPUT").unwrap()
}

pub async fn master_main() -> Result<(), Box<dyn std::error::Error>> {
    let self_ip = get_self_ip();
    let connected_ips = get_ip_list(&self_ip).await;
    let temp: Vec<&str> = connected_ips.iter().map(|s| s.as_str()).collect();
    let mut jm = JobManager::new(self_ip, &temp).await?;
    let mut pipeline: VecDeque<PipelineTask> = VecDeque::new();
    pipeline.push_back(PipelineTask {
        task_type: TaskType::Map,
    });
    pipeline.push_back(PipelineTask {
        task_type: TaskType::Shuffle,
    });
    pipeline.push_back(PipelineTask {
        task_type: TaskType::Reduce,
    });
    let wordcount = Job::new("matrix", &get_input_file(), pipeline);
    jm.queue_job(wordcount);
    println!("Starting...");
    jm.start().await;
    Ok(())
}
