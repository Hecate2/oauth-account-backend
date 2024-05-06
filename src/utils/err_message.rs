use serde::Serialize;

#[derive(Serialize)]
pub struct ErrMessage {
    pub err: String,
    pub public_key: Option<String>,
}
