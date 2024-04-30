use actix_web::{Either, HttpRequest, HttpResponse};
use super::err_message::ErrMessage;

// Get XXX from header_key: XXX
pub fn get_bearer_token(req: HttpRequest, header_key: &str) -> Either<String, HttpResponse>{
    let auth_header = match req.headers().get(header_key) {
        Some(authen_header) => authen_header,
        None => {
            return Either::Right(HttpResponse::Unauthorized().content_type("application/json").json(ErrMessage{err: "No auth".to_string()}));
        }
    };
    let auth_str = auth_header.to_str().unwrap_or("");
    return Either::Left(auth_str.to_string());
}