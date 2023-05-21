use serde_json::{map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;

/** @tables: paths must be stored as absolute paths */
pub struct DataBase {
    table: HashMap<PathBuf, Value>,
    dir: String,
}

impl DataBase {
    pub fn open(dir: PathBuf) -> DataBase {
        let mut table = HashMap::new();
        let mut dir_entry = fs::read_dir(&dir).expect("Database not found");
        while let Some(Ok(path)) = dir_entry.next() {
            let path = path.path();
            if !path.is_file() {
                continue;
            }
            if let Ok(mut file) = File::open(&path) {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .expect("File unable to write to string");
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    table.insert(path, json);
                }
            }
        }

        DataBase {
            table,
            dir: dir.into_os_string().into_string().unwrap(),
        }
    }

    pub fn add(&mut self, data: map::Map<String, Value>) -> Result<(), Box<dyn Error>> {
        let id = data
            .get("_id")
            .expect("json invalid, id not found")
            .as_str()
            .unwrap()
            .to_string();
        let json = Value::Object(data);
        let path = PathBuf::from(format!("{}/{id}.json", self.dir));
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &json)?;
        let path = path.canonicalize()?;
        self.table.insert(path, json);
        Ok(())
    }

    pub fn delete(&mut self, id: String) -> Result<(), Box<dyn Error>> {
        let path = PathBuf::from(format!("{}/{id}.json", self.dir)).canonicalize()?;
        self.table.remove(&path).ok_or("path not found in table")?;
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn query(self, args: Vec<String>) -> Vec<Value> {
        let mut results = vec![];
        if args.contains(&"*".to_string()) {
            return self.table.into_values().collect();
        }
        for json in self.table.into_values() {
            let json = json.as_object().unwrap();
            let mut result = map::Map::new();
            args.iter().for_each(|arg| {
                if let Some((k, v)) = json.get_key_value(arg) {
                    result.insert(k.clone(), v.clone());
                }
            });
            if !result.is_empty() {
                results.push(Value::Object(result));
            }
        }
        results
    }

    pub fn query_id(&self, id: String) -> Option<&Value> {
        if let Ok(path) = PathBuf::from(format!("{}/{id}.json", self.dir)).canonicalize() {
            let json = self.table.get(&path)?;
            return Some(json);
        }
        None
    }

    pub fn modify(&mut self, id: String, field: String, val: Value) -> Result<(), Box<dyn Error>> {
        let path = PathBuf::from(format!("{}/{id}.json", self.dir)).canonicalize()?;
        let value = self
            .table
            .get_mut(&path)
            .ok_or("id not found in database")?;
        let json = value
            .as_object_mut()
            .expect("none object type was passed to the database");
        json.entry(field)
            .and_modify(|v| *v = val.clone())
            .or_insert(val);
        let mut file = OpenOptions::new().write(true).truncate(true).open(&path)?;
        let val_str = serde_json::to_string(value)?;
        file.write(val_str.as_bytes())?;
        Ok(())
    }
}
