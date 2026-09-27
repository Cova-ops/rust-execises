mod test_rcu_cache {
    use rust_execises::rcu_cache::RCU;
    use std::sync::Arc;

    #[test]
    fn new_cache_is_empty() {
        let cache = RCU::<u64, u64>::new();

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn insert_and_get_value() {
        let cache = RCU::new();

        cache.insert(1, 100);

        let value = cache.get(&1);

        assert_eq!(value.as_deref(), Some(&100));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn get_nonexistent_key_returns_none() {
        let cache = RCU::<u64, u64>::new();

        assert!(cache.get(&1).is_none());
    }

    #[test]
    fn inserting_same_key_replaces_value() {
        let cache = RCU::new();

        cache.insert(1, 100);
        cache.insert(1, 200);

        assert_eq!(cache.get(&1).as_deref(), Some(&200));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn insert_returns_previous_value() {
        let cache = RCU::new();

        let first = cache.insert(1, 100);
        let second = cache.insert(1, 200);

        assert!(first.is_none());
        assert_eq!(second.as_deref(), Some(&100));
    }

    #[test]
    fn remove_returns_value_and_removes_it() {
        let cache = RCU::new();

        cache.insert(1, 100);

        let removed = cache.remove(&1);

        assert_eq!(removed.as_deref(), Some(&100));
        assert!(cache.get(&1).is_none());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn fetched_value_survives_replacement() {
        let cache = RCU::new();

        cache.insert(1, 100);

        let old_value = cache.get(&1).unwrap();

        cache.insert(1, 200);

        assert_eq!(*old_value, 100);
        assert_eq!(*cache.get(&1).unwrap(), 200);
    }

    #[test]
    fn fetched_value_survives_removal() {
        let cache = RCU::new();

        cache.insert(1, 100);

        let value = cache.get(&1).unwrap();

        cache.remove(&1);

        assert_eq!(*value, 100);
        assert!(cache.get(&1).is_none());
    }

    #[test]
    fn get_returns_shared_ownership_of_same_value() {
        let cache = RCU::new();

        cache.insert(1, 100);

        let first = cache.get(&1).unwrap();
        let second = cache.get(&1).unwrap();

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn concurrent_inserts() {
        const THREADS: u64 = 10;
        const INSERTS_PER_THREAD: u64 = 1_000;

        let cache = Arc::new(RCU::new());

        let mut handles = Vec::new();

        for thread_id in 0..THREADS {
            let cache = Arc::clone(&cache);

            let handle = std::thread::spawn(move || {
                let start = thread_id * INSERTS_PER_THREAD;
                let end = start + INSERTS_PER_THREAD;

                for key in start..end {
                    cache.insert(key, key);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(cache.len(), (THREADS * INSERTS_PER_THREAD) as usize);

        for key in 0..(THREADS * INSERTS_PER_THREAD) {
            assert_eq!(cache.get(&key).as_deref(), Some(&key));
        }
    }
}
