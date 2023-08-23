use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::object::Object;

#[derive(Clone)]
pub struct Store {
    pub objects: Arc<RwLock<HashMap<String, Object>>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            objects: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
