use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction{
    pub id:u64,
}

impl Transaction{
    pub fn to_string(&self) ->String{
        format!("{}", self.id)
    }
    pub fn to_json(&self) -> String{
        serde_json::to_string(&self).unwrap()
    }
}