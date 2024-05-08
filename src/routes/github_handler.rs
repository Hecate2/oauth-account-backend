use actix_web::{error::{ErrorBadRequest, HttpError}, get, post, web::{self, head}, Either, HttpRequest, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityOrSelect, EntityTrait, QueryFilter, SelectColumns, Statement};
use sea_orm::ActiveValue::{Set, NotSet, Unchanged};
use crate::utils::{app_state::{self, AppState}, auth::get_bearer_token, err_message::ErrMessage};
use entity::user;
use reqwest;
use std::sync::Arc;
use std::error::Error;
use crate::crypto::bitcoin_keypair::BitcoinKeypair;

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
    let private_key = match
        user::Entity::find()
        .filter(user::Column::GithubId.eq(github_id))
        .select_column(user::Column::PrivateKey)
        .one(db_pool)
        .await {
            Ok(v) => match v {
                Some(s) => s,
                None => return HttpResponse::BadRequest().content_type("application/json").json(ErrMessage{err: "Not registered".to_string(), public_key: None}),
            },
            Err(e) => return HttpResponse::InternalServerError().content_type("application/json").json(ErrMessage{err: e.to_string(), public_key: None}),
        };
    HttpResponse::Ok().json(private_key)
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
    let private_key = match
        user::Entity::find()
        .filter(user::Column::GithubId.eq(github_id.to_owned()))
        .select_column(user::Column::PrivateKey)
        .one(db_pool)
        .await {
            Ok(v) => match v {
                Some(s) => return HttpResponse::BadRequest().content_type("application/json").json(ErrMessage{err: "Already registered".to_string(), public_key: None}),
                None => {
                    let bitcoin_keypair = BitcoinKeypair::new();
                    let user_db = user::ActiveModel {
                        private_key: Set(bitcoin_keypair.secret_key_wif.to_owned()),
                        github_id: Set(Some(github_id)),
                        ..Default::default()
                    };
                    match user_db.insert(db_pool).await {
                        Ok(i) => i,
                        Err(e) => return HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: e.to_string(), public_key: None}),                
                    };
                    bitcoin_keypair.public_key
                },
            },
            Err(e) => return HttpResponse::InternalServerError().content_type("application/json").json(ErrMessage{err: e.to_string(), public_key: None}),
        };
    HttpResponse::Ok().json(private_key)
}

pub fn config(config: &mut web::ServiceConfig){
    config
    .service(
        web::scope("/github")
        .service(get_public_key)
        .service(create_account)
    );
}