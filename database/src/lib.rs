use deadpool_diesel::postgres::{Manager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub mod models;
mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub async fn new_pool() -> Pool {
    // create connection pool
    let manager = Manager::new("postgresql://user:secret@localhost".to_string(), deadpool_diesel::Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    // apply pending migrations
    run_migrations(&pool).await;

    return pool
}

// Function to run database migrations
async fn run_migrations(pool: &Pool) {
    let conn = pool.get().await.unwrap();
    conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await
        .unwrap()
        .unwrap();
}