use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MasterMessage {
    MapDirective {
        id: usize,
        input_file: String,
        output_file: String,
    },
    ReduceDirective {
        id: usize,
        input_file: String,
        output_file: String,
    },
    ShuffleDirective {
        id: usize,
        input_file: String,
        splits: usize,
    },
}
impl MasterMessage {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        let msg: String = serde_json::to_string(&self).unwrap();
        return Vec::from(msg);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SlaveMessage {
    Done(usize),
}

impl SlaveMessage {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        let msg: String = serde_json::to_string(&self).unwrap();
        return Vec::from(msg);
    }
}
