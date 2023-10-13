pub trait CredStore {
    fn add(&mut self, key: String, value: String) -> &mut Self;
    fn get(&self, key: &str) -> Option<&String>;
    fn clear(&mut self) -> &mut Self;
    fn keys_present(&self, keys: &[String]) -> bool;
    fn load(&self) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn save(&self) -> Result<(), std::io::Error>;
    fn delete(&self) -> Result<(), std::io::Error>;
}
