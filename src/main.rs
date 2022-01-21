use actix_cors::Cors;
use actix_web::{
    get, middleware::Logger, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use keyplace::{AgentKey, CustodialAgentKey, PassKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::str;
use std::sync::Mutex;

async fn greet(req: HttpRequest) -> impl Responder {
    println!("greet hit");
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[derive(Serialize, Deserialize)]
struct SaveKeyInput {
    email: String,
    custodial_key: CustodialAgentKey,
}

#[post("/save_key")]
async fn save_key(
    req_body: String,
    user_info_data: web::Data<Mutex<HashMap<String, UserData>>>,
) -> Result<impl Responder, Error> {
    println!("save_key hit, req_body: {}", req_body);
    let json_result: Result<SaveKeyInput, serde_json::Error> = serde_json::from_str(&req_body);
    match json_result {
        Ok(email_and_key) => {
            let user_info_ref = user_info_data.get_ref();
            let mut user_info = user_info_ref.lock().unwrap();
            user_info.insert(
                email_and_key.email,
                UserData {
                    custodial_key: email_and_key.custodial_key,
                },
            );

            Ok(HttpResponse::Ok().finish())
        }
        Err(_) => Ok(HttpResponse::BadRequest().finish()),
    }
}

#[derive(Serialize, Deserialize)]
struct GetKeyInput {
    email: String,
}

#[post("/get_key")]
async fn get_key(
    req_body: String,
    user_info_data: web::Data<Mutex<HashMap<String, UserData>>>,
) -> Result<impl Responder, Error> {
    println!("get_key hit, req_body: {}", req_body);
    let json_result: Result<GetKeyInput, serde_json::Error> = serde_json::from_str(&req_body);
    match json_result {
        Ok(get_key_input) => {
            let user_info_ref = user_info_data.get_ref();
            let user_info = user_info_ref.lock().unwrap();
            if let Some(custodial_key) = user_info.get(&get_key_input.email) {
                println!("the cust key {:?}", custodial_key);
                Ok(HttpResponse::Ok().json(json!(custodial_key)))
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        Err(_) => Ok(HttpResponse::BadRequest().finish()),
    }
}

#[derive(Serialize, Deserialize)]
struct EmailStruct {
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserData {
    custodial_key: CustodialAgentKey,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // SESSION TABLE
    let user_keys: HashMap<String, UserData> = HashMap::new();
    let user_keys_data = web::Data::new(Mutex::new(user_keys));
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(user_keys_data.clone())
            .service(save_key)
            .service(get_key)
            .route("/greet", web::get().to(greet))
            .route("/greet/{name}", web::get().to(greet))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
