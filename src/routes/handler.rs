pub trait OAuthHandler {
    fn get_account_id(token: &str) -> String;
}