use std::{
    collections::HashMap,
    fmt,
    fs::{self, File},
    io::{self, BufReader},
    path::PathBuf,
    time::SystemTime,
};

use jsonpath_rust::JsonPathQuery;
use serde_json::Value;

pub struct Cache {
    base_path: PathBuf,
    map: HashMap<String, Value>,
    mtimes: HashMap<String, SystemTime>,
}

#[derive(Debug)]
pub enum CacheError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    KeyNotFound(String),
}

impl From<io::Error> for CacheError {
    fn from(e: io::Error) -> Self {
        CacheError::IoError(e)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(e: serde_json::Error) -> Self {
        CacheError::JsonError(e)
    }
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonError(e) => write!(f, "cannot read a content from json file. {}", e),
            Self::IoError(e) => write!(f, "cannot read a json file. {}", e),
            Self::KeyNotFound(key) => write!(f, "key does not found. key: {}", key),
        }
    }
}

impl Cache {
    pub fn new(base_path: String) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            map: HashMap::new(),
            mtimes: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, scope: String) -> Result<(), CacheError> {
        let path = PathBuf::from("/cwd")
            .join(&self.base_path)
            .join(format!("{}.json", scope));
        let path_string = path.to_str().unwrap().to_string();
        let modified = fs::metadata(&path)?.modified()?;

        if let Some(mtime) = self.mtimes.get(&path_string) {
            if *mtime == modified {
                return Ok(());
            }
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json = serde_json::from_reader(reader)?;

        self.map.insert(scope, json);
        self.mtimes.insert(path_string, modified);

        Ok(())
    }

    pub fn get(&self, filename: String, key: String) -> Result<String, CacheError> {
        let Some(json) = self.map.get(&filename) else {
            return Err(CacheError::KeyNotFound(key));
        };

        let key = format!("$.{}", key.as_str());
        let Ok(Value::Array(array)) = json.clone().path(&key) else {
            return Err(CacheError::KeyNotFound(key));
        };

        if let Some(value) = array.get(0) {
            Ok(value.as_str().unwrap().to_string())
        } else {
            Err(CacheError::KeyNotFound(key))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use super::*;

    #[test]
    fn test_cache() {
        let base_path = current_dir()
            .unwrap()
            .join(PathBuf::from("fixture/json"))
            .to_str()
            .unwrap()
            .to_string();

        let mut cache = Cache::new(base_path);

        // Test adding file to cache
        assert!(cache.add_file("noArgs".to_string()).is_ok());

        // Test getting value from cache
        assert_eq!(
            cache.get("noArgs".to_string(), "a".to_string()).unwrap(),
            "test1"
        );

        // Test getting non-existent key
        assert!(cache
            .get("noArgs".to_string(), "notExists".to_string())
            .is_err());

        // Test getting value from non-existent file
        assert!(cache.get("notExists".to_string(), "a".to_string()).is_err());
    }
}
