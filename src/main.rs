use serde_json::{map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;

const DB_DIR: &str = "./database";

/** @table: paths must be stored as absolute paths */
struct DataBase {
    table: HashMap<PathBuf, Value>,
}

impl DataBase {
    fn open(dir: PathBuf) -> DataBase {
        let mut table = HashMap::new();
        let mut dir_entry = fs::read_dir(dir).expect("Database not found");
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
        DataBase { table }
    }

    fn add(&mut self, data: map::Map<String, Value>) -> Result<(), Box<dyn Error>> {
        let id = data
            .get("_id")
            .expect("json invalid _id not found")
            .as_str()
            .unwrap()
            .to_string();
        let json = Value::Object(data);
        let path = PathBuf::from(format!("{DB_DIR}/{id}.json"));
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &json)?;
        let path = path.canonicalize()?;
        self.table.insert(path, json);
        Ok(())
    }

    fn delete(&mut self, id: String) {
        if let Ok(path) = PathBuf::from(format!("{DB_DIR}/{id}.json")).canonicalize() {
            self.table.remove(&path);
            fs::remove_file(path).expect("schrodinger's files");
        }
    }

    fn search(self, args: Vec<String>) -> Vec<Value> {
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
}

fn main() {
    let path = PathBuf::from(DB_DIR).canonicalize().unwrap();
    let mut database = DataBase::open(path);
    let a = database.search(vec![]);
    println!("{a:?}");
}
