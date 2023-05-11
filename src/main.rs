use serde_json::{map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;

const DB_DIR: &str = "./database";

struct DataBase {
    table: HashMap<PathBuf, Value>,
}

impl DataBase {
    fn open(dir: String) -> DataBase {
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
        let path = PathBuf::from(format!("{}/{id}.json", DB_DIR));
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &json)?;
        self.table.insert(path, json);
        Ok(())
    }

    fn delete(&mut self, id: String) {}
}

fn main() {
    let mut x = DataBase::open(DB_DIR.to_string());
    let mut m = serde_json::Map::new();
    m.insert("_id".to_string(), Value::String("1".to_string()));
    x.add(m).unwrap();
    println!("{:?}", x.table)
}
