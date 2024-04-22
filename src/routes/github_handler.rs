use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Either};
use sea_orm::{ConnectionTrait, Statement};
use crate::utils::auth::get_bearer_token;
use serde::Serialize;

#[derive(Serialize)]
struct ErrMessage {
    err: String,
}

#[get("")]
// Authorizaion: Bearer TOKEN
pub async fn greet(req: HttpRequest) -> impl Responder {
    let auth: String = match get_bearer_token(req) {
        Either::Right(errResp) => return errResp,
        Either::Left(token) => token,
    };
    // TODO: call github API with the token and get user id
    HttpResponse::Ok().body("OK")
}

pub fn config(config: &mut web::ServiceConfig){
    config
    .service(
        web::scope("/github")
        .service(greet)
    );
}