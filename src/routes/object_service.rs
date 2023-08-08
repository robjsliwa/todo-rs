use crate::models::Object;
use async_trait::async_trait;

#[async_trait]
pub trait ObjectService {
    async fn get_object(&self, id: &str) -> Option<Object>;
    async fn insert_object(&self, object: Object) -> bool;
    async fn update_object(&self, object: Object) -> bool;
    async fn delete_object(&self, id: &str) -> bool;
}
