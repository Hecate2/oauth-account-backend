use migration::backend;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema};
use entity::{self, user};

pub async fn create_tables_if_not_exists(db: &DatabaseConnection) {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let mut table_create_statement = schema.create_table_from_entity(user::Entity);
    table_create_statement.if_not_exists();
    if let Err(table_create_result) = db.execute(backend.build(&table_create_statement)).await {
        println!("{}", table_create_result);
    }
    let mut index_create_statements = schema.create_index_from_entity(user::Entity);
    for s in &mut index_create_statements {
        s.if_not_exists();
        if let Err(index_create_result) = db.execute(backend.build(s)).await {
            println!("{}", index_create_result);
        }
    }
}