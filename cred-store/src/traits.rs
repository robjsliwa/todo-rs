pub trait CredStore {
    fn add(&mut self, key: String, value: String);
    fn get(&self, key: &str) -> Option<&String>;
    fn clear(&mut self);
    fn keys_present(&self, keys: &[String]) -> bool;
}
