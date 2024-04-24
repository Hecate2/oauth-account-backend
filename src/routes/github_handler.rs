use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Either};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityOrSelect, EntityTrait, QueryFilter, SelectColumns, Statement};
use crate::utils::{app_state::{self, AppState}, auth::get_bearer_token};
use entity::user;
use serde::Serialize;

async fn get_account_id(token: String) -> String {
    // TODO: call github API with the token and get user id
    return token.to_string();
}

#[get("")]
// Authorizaion: Bearer TOKEN
pub async fn get_public_key(req: HttpRequest, db: web::Data<DatabaseConnection>) -> impl Responder {
    let token: String = match get_bearer_token(req) {
        Either::Right(errResp) => return errResp,
        Either::Left(token) => token,
    };
    let github_id = get_account_id(token).await;
    let conn = db.as_ref();
    let public_key = 
        user::Entity::find()
        .filter(user::Column::GithubId.eq(github_id))
        .select_column(user::Column::PublicKey)
        .one(conn)
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