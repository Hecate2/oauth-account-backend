use actix_web::{error::{ErrorBadRequest, HttpError}, get, post, web::{self, head}, Either, HttpRequest, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityOrSelect, EntityTrait, QueryFilter, SelectColumns, Statement};
use sea_orm::ActiveValue::{Set, NotSet, Unchanged};
use crate::utils::{app_state::{self, AppState}, auth::get_bearer_token, err_message::ErrMessage};
use entity::user;
use entity::private_key;
use reqwest;
use std::sync::Arc;
use std::error::Error;
use crate::crypto::bitcoin_keypair;

async fn get_account_id(token: String) -> Result<String, Box<dyn Error>> {
    // TODO: call github API with the token and get user id
    let client = reqwest::ClientBuilder::new().build()?;
    let res = client.get("https://api.github.com/user")
    .header("User-Agent", "reqwest")
    .header("Authorization", "Bearer ".to_owned() + &token)
    .send().await?;
    // let body = res.text().await?;
    // println!("Body:\n{}", body);
    let json_body: serde_json::Value = res.json().await?;
    let id = match json_body.get("id") {
        Some(v) => v,
        None => return Err(Box::new(ErrorBadRequest("No id returned from github"))),
    };
    Ok(id.to_string())
}

const HEADER_KEY: &str = "X-Github";

#[get("")]
// X-Github: gho...
pub async fn get_public_key(req: HttpRequest, state: web::Data<Arc<AppState>>) -> impl Responder {
    let token: String = match get_bearer_token(req, HEADER_KEY) {
        Either::Right(err_resp) => return err_resp,
        Either::Left(token) => token,
    };
    let github_id = match get_account_id(token).await {
        Ok(i) => i,
        Err(e) => return HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "Invalid token".to_string(), public_key: None}),
    };
    let db_pool = &state.db;
    let public_key = match
        user::Entity::find()
        .filter(user::Column::GithubId.eq(github_id))
        .select_column(user::Column::PublicKey)
        .one(db_pool)
        .await {
            Ok(v) => match v {
                Some(s) => s,
                None => return HttpResponse::BadRequest().content_type("application/json").json(ErrMessage{err: "Not registered".to_string(), public_key: None}),
            },
            Err(e) => return HttpResponse::InternalServerError().content_type("application/json").json(ErrMessage{err: e.to_string(), public_key: None}),
        };
    HttpResponse::Ok().json(public_key)
}

#[post("")]
// X-Github: gho...
pub async fn create_account(req: HttpRequest, state: web::Data<Arc<AppState>>) -> impl Responder {
    let token: String = match get_bearer_token(req, HEADER_KEY) {
        Either::Right(err_resp) => return err_resp,
        Either::Left(token) => token,
    };
    let github_id = match get_account_id(token).await {
        Ok(i) => i,
        Err(e) => return HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "Invalid token".to_string(), public_key: None}),
    };
    let db_pool = &state.db;
    let public_key = match
        user::Entity::find()
        .filter(user::Column::GithubId.eq(github_id.to_owned()))
        .select_column(user::Column::PublicKey)
        .one(db_pool)
        .await {
            Ok(v) => match v {
                Some(s) => return HttpResponse::BadRequest().content_type("application/json").json(ErrMessage{err: "Already registered".to_string(), public_key: None}),
                None => {
                    let secp = Secp256k1::new();
                    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
                    let pk_sha256 = Sha256Hash::hash(&public_key.serialize()).to_byte_array();
                    let mut pk_ripemp160 = Ripemp160Hash::hash(&pk_sha256).to_byte_array().to_vec();
                    pk_ripemp160.insert(0, 0x00);
                    let checksum = &Sha256Hash::hash(
                        &Sha256Hash::hash(&pk_ripemp160)
                            .to_byte_array()
                    )[..4];
                    let secret_key = SecretKey::random(&mut rand_core::OsRng);
                    let secret_key_bytes = secret_key.to_bytes();
                    let wif = bs58::encode(secret_key_bytes).with_check().into_string();
                    let public_key = secret_key.public_key().to_sec1_bytes();
                    let public_key_hex_string = public_key.iter()
                        .map(|b| format!("{:02x}", b).to_string())
                        .collect::<Vec<String>>()
                        .join("");
                    let key_pair_db = private_key::ActiveModel {
                        private_key: Set(wif),
                        public_key: Set(public_key_hex_string.to_owned()),
                    };
                    match key_pair_db.insert(db_pool).await {
                        Ok(i) => i,
                        Err(e) => return HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: e.to_string(), public_key: None}),                
                    };

                    let user_db = user::ActiveModel {
                        public_key: Set(public_key_hex_string.to_owned()),
                        github_id: Set(github_id),
                        ..Default::default()
                    };
                    match user_db.insert(db_pool).await {
                        Ok(i) => i,
                        Err(e) => return HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: e.to_string(), public_key: None}),                
                    };
                    public_key_hex_string
                },
            },
            Err(e) => return HttpResponse::InternalServerError().content_type("application/json").json(ErrMessage{err: e.to_string(), public_key: None}),
        };
    HttpResponse::Ok().json(public_key)
}

pub fn config(config: &mut web::ServiceConfig){
    config
    .service(
        web::scope("/github")
        .service(get_public_key)
        .service(create_account)
    );
}