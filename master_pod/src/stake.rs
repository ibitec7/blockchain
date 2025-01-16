use serde::{Serialize as SerdeSerialize, Deserialize};
use serde_json::{from_str, to_string};

#[derive(SerdeSerialize, Deserialize, Clone)]
pub struct Stake {
    pub node_id: String,
    pub stake: f64,
}

impl Stake {
    pub fn serialize(&self) -> String {
        let json_string = to_string(&self).expect("Failed to serialize");
        json_string
    }

    pub fn deserialize(json_str: String) -> Self {
        let stake: Stake = from_str(&json_str).expect("Failed to deserialize");
        stake
    }
}

#[derive(SerdeSerialize, Deserialize, Clone)]
pub struct Validator {
    pub node_id: String,
    pub public_key: String
}

impl Validator {
    pub fn from_stake(stake: &Stake) -> Self {
        Validator { node_id: stake.node_id.clone(), public_key: stake.node_id.clone() }
    }

    pub fn serialize(&self) -> String {
        let json_string = to_string(&self).expect("Failed to serialize");
        json_string
    }

    pub fn deserialize(json_str: String) -> Self {
        let validator: Validator = from_str(&json_str).expect("Failed to deserialize");
        validator
    }
}