mod database;

use crate::database::DataBase;
use std::path::PathBuf;

const DB_DIR: &str = "./database";

fn main() {
    let path = PathBuf::from(DB_DIR).canonicalize().unwrap();
    let mut database = DataBase::open(path);
    let x = database.query_id("1tabl9587".to_string()).unwrap();
    println!("{:?}", x);
}
