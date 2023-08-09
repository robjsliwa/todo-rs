use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: String,
    pub value: serde_json::Value,
}

pub type DB = Arc<RwLock<HashMap<String, Object>>>;
