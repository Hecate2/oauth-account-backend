use actix_web::{Either, HttpRequest, HttpResponse};
use super::err_message::ErrMessage;

pub fn get_bearer_token(req: HttpRequest) -> Either<String, HttpResponse>{
    let auth_header = match req.headers().get("Authorization") {
        Some(authen_header) => authen_header,
        None => {
            return Either::Right(HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "No auth".to_string()}));
        }
    };
    let auth_str = auth_header.to_str().unwrap_or("");
    let auth_len = auth_str.len();
    if auth_len <= "bearer ".len() {
        return Either::Right(HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "Malformed auth".to_string()}));
    }
    let bearer = auth_str[0.."bearer".len()].to_lowercase();
    if !bearer.starts_with("bearer") {
        return Either::Right(HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "Malformed auth".to_string()}));
    }
    let raw_token = auth_str["bearer ".len()..auth_str.len()].trim();
    return Either::Left(raw_token.to_string());
}