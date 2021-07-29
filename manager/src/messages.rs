use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MasterMessage {
    MapDirective { input_file: String },
    ReduceDirective { input_file: String },
    ShuffleDirective { input_files: Vec<String> },
}
impl MasterMessage {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        let msg: String = serde_json::to_string(&self).unwrap();
        return Vec::from(msg);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SlaveMessage {
    Done,
}

impl SlaveMessage {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        let msg: String = serde_json::to_string(&self).unwrap();
        return Vec::from(msg);
    }
}
