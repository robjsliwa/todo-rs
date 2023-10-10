pub trait CredStore {
    fn add(&mut self, key: String, value: String) -> &mut Self;
    fn get(&self, key: &str) -> Option<&String>;
    fn clear(&mut self) -> &mut Self;
    fn keys_present(&self, keys: &[String]) -> bool;
}
