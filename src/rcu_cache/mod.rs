use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct RCU<K, V>
where
    K: Eq + std::hash::Hash,
{
    inner: Mutex<HashMap<K, Arc<V>>>,
}

impl<K, V> RCU<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<Arc<V>> {
        let mut hash = self.inner.lock().unwrap();
        hash.insert(key, Arc::new(value))
    }

    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        let hash = self.inner.lock().unwrap();
        hash.get(key).map(|v| Arc::clone(v))
    }

    pub fn remove(&self, key: &K) -> Option<Arc<V>> {
        let mut hash = self.inner.lock().unwrap();
        hash.remove(key)
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
}

pub fn run(_: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Clear screen
    let command = if cfg!(target_os = "windows") {
        Some("cls")
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        Some("clear")
    } else {
        None
    };

    match command {
        Some(v) => {
            std::process::Command::new(v).status()?;
        }
        _ => println!("Error at cleaning screen"),
    }

    println!(
        r#"
RCU Cache is a library/data-structure project, not a CLI application.

Run:
    cargo test rcu_cache

to execute its test suite.
        "#
    );

    Ok(())
}
