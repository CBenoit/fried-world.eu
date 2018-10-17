use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;
use std::time::Instant;

struct Item_<V> {
    content: V,
    time: Instant,
}

/// A structure that cache values associated to keys for a minimal given duration.
/// Checks are performed when accessed.
pub struct Cacher<K, V> {
    data: HashMap<K, Item_<V>>,
    cache_duration: Duration,
    min_clean_interval: Duration,
    last_clean_instant: Instant,
}

impl<K, V> Cacher<K, V>
where
    K: Eq + Hash,
{
    /// create a new Cacher whose cached values lasts a least `cache_duration` and
    /// whose minimal clean interval is `min_clean_interval`.
    pub fn new(cache_duration: Duration, min_clean_interval: Duration) -> Cacher<K, V> {
        Cacher {
            data: HashMap::new(),
            cache_duration,
            min_clean_interval,
            last_clean_instant: Instant::now(),
        }
    }

    /// Retrieve the value associated to the given key.
    /// None if no value has been inserted yet or if the
    /// value is expired. Use `get_or_insert` to insert.
    pub fn get(&self, key: &K) -> Option<&V> {
        let item = self.data.get(key)?;
        if Instant::now() - item.time < self.cache_duration {
            Some(&item.content)
        } else {
            None
        }
    }

    /// Retrieve the value associated to the given key.
    /// If no value has been inserted yet, the producer function
    /// is used to create the value which is stored and returned.
    pub fn get_or_insert<F>(&mut self, key: K, producer: F) -> &V
    where
        F: FnOnce() -> V,
    {
        if Instant::now() - self.last_clean_instant >= self.min_clean_interval {
            self.cleanup();
        }

        let item = self.data.entry(key).or_insert_with(|| Item_ {
            content: producer(),
            time: Instant::now(),
        });

        &item.content
    }

    fn cleanup(&mut self) {
        self.last_clean_instant = Instant::now();

        let now = Instant::now();
        let cache_duration = self.cache_duration;
        self.data.retain(|_, v| now - v.time < cache_duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn get_cacher() -> Cacher<&'static str, &'static str> {
        Cacher::new(Duration::from_millis(200), Duration::from_millis(100))
    }

    #[test]
    fn insert_and_get_once() {
        let mut cacher = get_cacher();

        let compute_value = || "value";
        let panic = || panic!("cacher was called where it should not have been.");

        assert_eq!(*cacher.get_or_insert("key", compute_value), "value");
        assert_eq!(*cacher.get_or_insert("key", panic), "value");
        assert_eq!(cacher.get(&"key"), Some(&"value"));
    }

    #[test]
    fn insert_multiple_values() {
        let mut cacher = get_cacher();

        let compute_value_1 = || "value1";
        let compute_value_2 = || "value2";
        let panic = || panic!("cacher was called where it should not have been.");

        assert_eq!(*cacher.get_or_insert("key1", compute_value_1), "value1");
        assert_eq!(*cacher.get_or_insert("key1", panic), "value1");
        assert_eq!(*cacher.get_or_insert("key2", compute_value_2), "value2");
        assert_eq!(*cacher.get_or_insert("key2", panic), "value2");
        assert_eq!(*cacher.get_or_insert("key1", panic), "value1");
        assert_eq!(*cacher.get_or_insert("key2", panic), "value2");
    }

    #[test]
    fn check_cleanup() {
        let mut cacher = get_cacher();

        let compute_value = || "value";

        cacher.get_or_insert("key1", compute_value);

        let old_last_clean_instant = cacher.last_clean_instant;
        thread::sleep(Duration::from_millis(150));
        cacher.get_or_insert("key2", compute_value);
        assert_ne!(
            old_last_clean_instant, cacher.last_clean_instant,
            "last_clean_instant should have been updated, but it's not."
        );

        assert_eq!(cacher.get(&"key2"), Some(&"value"));

        thread::sleep(Duration::from_millis(150));
        cacher.get_or_insert("key2", compute_value);
        assert!(
            !cacher.data.contains_key("key1"),
            "this key should not exist anymore."
        );

        assert_eq!(cacher.get(&"key1"), None);
    }
}
