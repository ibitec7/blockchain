use serde_json::{to_string, from_str};
use serde::{Serialize as SerdeSerialize,Deserialize};

#[derive(SerdeSerialize, Deserialize, Clone)]
pub struct User {
    pub user_id: String,
    pub balance: f64,
}

impl User {
    pub fn serialize(&self) -> String{
        let json_string = to_string(&self).unwrap();
        json_string
    }

    pub fn deserialize(json_string: &String) -> Self {
        let msg: User = from_str(json_string).unwrap();
        msg
    }
}