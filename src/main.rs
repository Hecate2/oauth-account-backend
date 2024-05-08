use actix_web::{get, middleware, web, App, HttpServer, Responder};
use entity::user;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use utils::app_state::AppState;
use std::sync::Arc;

mod utils;
mod routes;
mod init;
mod crypto;


#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    std::env::set_var("RUST_LOG", "debug");  // for dev only

    dotenv::dotenv().ok();
    env_logger::init();


    let port = (*utils::constants::PORT).clone();
    let address = (*utils::constants::ADDRESS).clone();
    let database_url = (*utils::constants::DATABASE_URL).clone();
    let app_state = AppState::new(&database_url).await;
    let arc_app_state = Arc::new(app_state);
    
    init::create_tables_if_not_exists(&arc_app_state.db, user::Entity).await;
    // init::create_tables_if_not_exists(&arc_app_state.db, private_key::Entity).await;
    // Migrator::up(&db, None).await.unwrap();

    HttpServer::new(move || {
        App::new()
        .wrap(middleware::NormalizePath::trim())
        .data(arc_app_state.clone())
        .wrap(middleware::Logger::default())
        .configure(routes::github_handler::config)
        .configure(routes::google_handler::config)
    })
    .bind((address, port))?
    .run()
    .await
}