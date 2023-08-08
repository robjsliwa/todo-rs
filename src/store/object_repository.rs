use crate::models::Object;
use crate::routes::object_service::ObjectService;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ObjectRepository {
    objects: Arc<RwLock<HashMap<String, Object>>>,
}

impl ObjectRepository {
    pub fn new() -> Self {
        ObjectRepository {
            objects: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl ObjectService for ObjectRepository {
    async fn get_object(&self, id: &str) -> Option<Object> {
        let read_objects = self.objects.read().await;
        read_objects.get(id).cloned()
    }

    async fn insert_object(&self, object: Object) -> bool {
        let mut write_objects = self.objects.write().await;
        write_objects.insert(object.id.clone(), object).is_some()
    }

    async fn update_object(&self, object: Object) -> bool {
        let mut write_objects = self.objects.write().await;
        write_objects.insert(object.id.clone(), object).is_some()
    }

    async fn delete_object(&self, id: &str) -> bool {
        let mut write_objects = self.objects.write().await;
        write_objects.remove(id).is_some()
    }
}
