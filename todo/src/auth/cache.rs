use crate::model::User;
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct UserCache {
    pub cache: LruCache<String, User>,
}

impl UserCache {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }
}
