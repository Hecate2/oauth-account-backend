use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use sea_orm::{ConnectionTrait, Statement};
use crate::utils::{api_response, app_state::{self, AppState}};
use serde::Serialize;

#[derive(Serialize)]
struct ErrMessage {
    err: String,
}

#[get("")]
pub async fn greet(req: HttpRequest) -> impl Responder {
    let authen_header = match req.headers().get("Authorization") {
        Some(authen_header) => authen_header,
        None => {
            return HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "No auth".to_string()});
        }
    };
    let authen_str = authen_header.to_str().unwrap_or("");
    if !authen_str.starts_with("bearer") && !authen_str.starts_with("Bearer") {
        return HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "Malformed auth".to_string()});
    }
    let raw_token = authen_str[6..authen_str.len()].trim();
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