mod database;

use crate::database::DataBase;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const DB_DIR: &str = "./database";
const IP_PORT: &str = "127.0.0.1:42069";

lazy_static::lazy_static! {
    static ref DB: Arc<RwLock<DataBase>> = Arc::new(RwLock::new(DataBase::new()));
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let path = PathBuf::from(DB_DIR).canonicalize().unwrap();
    if let Ok(mut database) = DB.write() {
        database.connect(path).unwrap();
    }
    HttpServer::new(move || {
        App::new()
            .route("/delete", web::delete().to(delete))
            .route("/add", web::post().to(add))
    })
    .bind(IP_PORT)?
    .run()
    .await
}

async fn add(json: web::Json<Value>) -> impl Responder {
    let json = json.0.as_object();
    if json.is_none() {
        return HttpResponse::BadRequest().body("value could not be processed");
    }
    let json = json.unwrap();
    if let Ok(mut database) = DB.write() {
        if let Err(e) = database.add(json.clone()) {
            return HttpResponse::BadRequest().body(e.to_string());
        }
        return HttpResponse::Ok().body("element added");
    }
    HttpResponse::BadRequest().body("unable to add element")
}

async fn delete(json: web::Json<Value>) -> impl Responder {
    let json = json.0.as_object();
    if json.is_none() {
        return HttpResponse::BadRequest().body("invalid format");
    }
    let json = json.unwrap();
    if let Ok(mut database) = DB.write() {
        let id = json.get("_id");
        if id.is_none() {
            return HttpResponse::BadRequest().body("format invalid");
        }
        let id = id.unwrap().as_str();
        if id.is_none() {
            return HttpResponse::BadRequest().body("format invalid");
        }
        if let Err(e) = database.delete(id.unwrap()) {
            return HttpResponse::BadRequest().body(e.to_string());
        }
        return HttpResponse::Ok().body("element removed");
    }
    HttpResponse::BadRequest().body("failed to delete element")
}
