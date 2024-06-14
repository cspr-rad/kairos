use deadpool_diesel::postgres::{Manager, Runtime};
use diesel::{prelude::*, select, sql_types::Text};
use tracing::{debug, info};

#[cfg(feature = "migrations")]
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub use deadpool_diesel::postgres::Pool;
pub use diesel::{insert_into, prelude};

pub mod errors;
pub mod schema;
pub mod transaction;

#[cfg(feature = "migrations")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn new(conn_str: &str) -> Result<Pool, errors::DBError> {
    let manager = Manager::new(conn_str, Runtime::Tokio1);
    debug!("Setup DB manager.");
    let conn_pool = Pool::builder(manager).max_size(8).build().unwrap();
    debug!("Initialized connection pool.");
    #[cfg(feature = "migrations")]
    run_migrations(&conn_pool).await;
    let conn = conn_pool.get().await?;
    let result = conn
        .interact(|conn| {
            let query = select("Hello world!".into_sql::<Text>());
            query.get_result::<String>(conn)
        })
        .await??;
    assert!(result == "Hello world!");
    info!("Created connection pool!");
    Ok(conn_pool)
}

// Function to run database migrations
#[cfg(feature = "migrations")]
async fn run_migrations(pool: &Pool) {
    let conn = pool.get().await.unwrap();
    conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await
        .unwrap()
        .unwrap();
}
