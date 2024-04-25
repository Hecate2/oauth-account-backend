use actix_web::{get, post, web::{self, head}, Either, HttpRequest, HttpResponse, Responder};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityOrSelect, EntityTrait, QueryFilter, SelectColumns, Statement};
use crate::utils::{app_state::{self, AppState}, auth::get_bearer_token, err_message::ErrMessage};
use entity::user;
use reqwest;
use std::sync::Arc;
use std::error::Error;
use serde::Serialize;

async fn get_account_id(token: String) -> Result<String, Box<dyn Error>> {
    // TODO: call github API with the token and get user id
    let client = reqwest::ClientBuilder::new().build()?;
    let res = client.get("https://api.github.com/user")
    .header("User-Agent", "reqwest")
    .header("Authorization", "Bearer ".to_owned() + &token)
    .send().await?;
    let body = res.text().await?;
    println!("Body:\n{}", body);
    Ok(body.to_string())
}

#[get("")]
// Authorizaion: Bearer TOKEN
pub async fn get_public_key(req: HttpRequest, state: web::Data<Arc<AppState>>) -> impl Responder {
    let token: String = match get_bearer_token(req) {
        Either::Right(err_resp) => return err_resp,
        Either::Left(token) => token,
    };
    let github_id = match get_account_id(token).await {
        Ok(i) => i,
        Err(e) => return HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "Invalid token".to_string()}),
    };
    let db_pool = &state.db;
    let public_key = 
        user::Entity::find()
        .filter(user::Column::GithubId.eq(github_id))
        .select_column(user::Column::PublicKey)
        .one(db_pool)
        .await
        .expect("Not registered").unwrap();
    HttpResponse::Ok().json(public_key)
}

pub fn config(config: &mut web::ServiceConfig){
    config
    .service(
        web::scope("/github")
        .service(get_public_key)
    );
}