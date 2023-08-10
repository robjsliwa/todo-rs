use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: String,
    pub value: serde_json::Value,
}
