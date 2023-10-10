use super::traits::CredStore;
use dirs;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::path::Path;

const CREDENTIALS_FILE: &str = ".credentials.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    data: HashMap<String, String>,
    file_name: String,
}

impl Credentials {
    pub fn new() -> Self {
        Credentials {
            data: HashMap::new(),
            file_name: CREDENTIALS_FILE.to_string(),
        }
    }

    pub fn set_file_name(mut self, file_name: String) -> Self {
        self.file_name = file_name;
        self
    }

    pub fn build(&self) -> Self {
        println!("file_name: {}", self.file_name);
        Credentials {
            data: self.data.clone(),
            file_name: self.file_name.clone(),
        }
    }

    pub fn load(&self) -> Result<Self, Error> {
        let store_path = match dirs::home_dir() {
            Some(path) => path.join(self.file_name.clone()),
            None => {
                return Err(Error::new(
                    std::io::ErrorKind::NotFound,
                    "Home directory not found",
                ))
            }
        };
        if Path::new(&store_path).exists() {
            let contents = fs::read_to_string(&store_path)?;
            let credentials: Credentials = serde_json::from_str(&contents)?;
            Ok(credentials)
        } else {
            Ok(Credentials {
                data: HashMap::new(),
                file_name: self.file_name.clone(),
            })
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        let store_path = match dirs::home_dir() {
            Some(path) => path.join(self.file_name.clone()),
            None => {
                return Err(Error::new(
                    std::io::ErrorKind::NotFound,
                    "Home directory not found",
                ))
            }
        };
        let contents = serde_json::to_string_pretty(&self)?;
        fs::write(store_path, contents)?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), Error> {
        let store_path = match dirs::home_dir() {
            Some(path) => path.join(self.file_name.clone()),
            None => {
                return Err(Error::new(
                    std::io::ErrorKind::NotFound,
                    "Home directory not found",
                ))
            }
        };
        if Path::new(&store_path).exists() {
            fs::remove_file(store_path)?;
        }
        Ok(())
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Credentials::new()
    }
}

impl CredStore for Credentials {
    fn add(&mut self, key: String, value: String) -> &mut Self {
        self.data.insert(key, value);
        self
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    fn clear(&mut self) -> &mut Self {
        self.data.clear();
        self
    }

    fn keys_present(&self, keys: &[String]) -> bool {
        keys.iter().all(|key| self.data.contains_key(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials() {
        let mut credentials = Credentials::new()
            .set_file_name(".test.json".to_string())
            .build()
            .load()
            .expect("Failed to load credentials");

        credentials.add("username".to_string(), "admin".to_string());
        credentials.add("password".to_string(), "12345".to_string());

        credentials.save().expect("Failed to save credentials");

        match credentials.get("username") {
            Some(value) => println!("Username: {}", value),
            None => println!("Username not found"),
        }

        assert!(credentials.keys_present(&["username".to_string(), "password".to_string()]));
        credentials.clear();
        credentials.delete().expect("Failed to delete credentials");
    }
}
