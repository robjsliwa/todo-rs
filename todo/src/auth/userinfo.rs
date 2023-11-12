use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UserInfo {
    pub sub: String,
    pub name: String,
    pub email: String,
}
