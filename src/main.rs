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

// // agent key is generated and is connected to the user's email
// // it contains a private and public halves of an asymmetric key
// let email = "yes@yes.com";
// let agentkey = AgentKey::create(Some(email.to_string()));
// println!("agent key 1: {:?}", agentkey);

// // the passkey is the key generated from a private password
// // neither the pw nor the passkey never need or should be sent anywhere
// let pw = "dank memes";
// let passkey = PassKey::new(pw);

// // the custodial key is generated from the agentkey and the passkey
// // it can be stored anywhere public as a way to recover the agentkey with a new passkey
// let custkey = agentkey.custodial_key(passkey);
// println!("cust key: {:?}", custkey);

// // a new passkey is created using the previous password
// let passkey2 = PassKey::new(pw);
// let agentkey2 = AgentKey::from_custodial_key(custkey.clone(), passkey2).unwrap();
// println!("agent key 2: {:?}", agentkey2.keypair);

// // this does not work. It will not create a new agent key.
// let new_pw = "dankest memes bruh";
// let newpasskey = PassKey::new(new_pw);
// let agentkey3 = AgentKey::from_custodial_key(custkey.clone(), newpasskey).unwrap();
// println!("agent key 3: {:?}", agentkey3.keypair);

// // this simplifies the process of the user having to remember an absurd private key

// // NOTES
// // A pw/passkey could be used multiple times for different keypairs
// // Or, a keypair could be stored multiple times with different passwords
// // a passkey and a new email, could be used to generate a
