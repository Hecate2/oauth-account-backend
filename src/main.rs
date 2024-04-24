use actix_web::{get, middleware, web, App, HttpServer, Responder};
use entity::{private_key, user};
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use utils::app_state::AppState;
use std::time::Duration;

mod utils;
mod routes;
mod init;



#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    dotenv::dotenv().ok();
    env_logger::init();


    let port = (*utils::constants::PORT).clone();
    let address = (*utils::constants::ADDRESS).clone();
    let database_url = (*utils::constants::DATABASE_URL).clone();
    let mut opt: ConnectOptions = ConnectOptions::new(database_url);
    opt.max_connections(32)
        .min_connections(4)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);
    let db: DatabaseConnection = Database::connect(opt).await.unwrap();
    init::create_tables_if_not_exists(&db, user::Entity).await;
    init::create_tables_if_not_exists(&db, private_key::Entity).await;
    // Migrator::up(&db, None).await.unwrap();

    HttpServer::new(move || {
        App::new()
        .wrap(middleware::NormalizePath::trim())
        .app_data(web::Data::new( AppState { db: db.clone() } ))
        .wrap(middleware::Logger::default())
        .configure(routes::github_handler::config)
    })
    .bind((address, port))?
    .run()
    .await
}