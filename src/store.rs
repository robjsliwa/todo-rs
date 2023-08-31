use crate::object::Object;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Store {
    pub objects: Arc<RwLock<HashMap<String, Object>>>,
    file_path: String,
}

impl Store {
    pub fn new(file_path: String) -> Self {
        Store {
            objects: Arc::new(RwLock::new(Self::load(&file_path))),
            file_path,
        }
    }

    fn load(file_path: &str) -> HashMap<String, Object> {
        match std::fs::read_to_string(file_path) {
            Ok(file) => serde_json::from_str(&file).unwrap_or_else(|_| {
                eprintln!("Failed to parse the JSON. Exiting...");
                process::exit(1);
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File not found, continue
                HashMap::new()
            }
            Err(e) => {
                eprintln!("An error occurred while reading the file: {}...", e);
                process::exit(1);
            }
        }
    }

    pub async fn shutdown(&self) -> std::io::Result<()> {
        let data = self.objects.read().await;
        let json = serde_json::to_string(&*data).expect("Failed to save data!");
        tokio::fs::write(&self.file_path, json).await
    }
}
