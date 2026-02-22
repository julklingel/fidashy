use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sample {
    pub sample1: String,
    pub sample2: String,

}


