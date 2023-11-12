use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub external_id: String,
    pub name: String,
    pub email: String,
    pub tenant_id: String,
}

impl User {
    pub fn new(external_id: String, name: String, email: String, tenant_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            external_id,
            name,
            email,
            tenant_id,
        }
    }
}
