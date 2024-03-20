mod domain;
mod config;
mod errors;
mod routes;
mod database;
mod handlers;
mod tasks;

use fern;
use chrono::Utc;
use tokio::signal;
use lazy_static::lazy_static;
use deadpool_diesel::postgres::{Manager, Pool};

use routes::app_router;

// Logging macro(s) for easy use
use log::info;
// Load config
lazy_static! {
    static ref CONFIG: config::Config = config::Config::read_config();
}
#[derive(Clone)]
pub struct AppState {
    pool: Pool,
}

#[tokio::main]
async fn main() {
    // Setup logging
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                Utc::now().to_rfc3339(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(CONFIG.log.level)
        .chain(if CONFIG.log.stdout {
            Box::new(std::io::stdout()) as Box<dyn std::io::Write + Send>
        } else {
            Box::new(std::io::sink())
        })
        .chain(fern::log_file(&CONFIG.log.file_output).expect("Error setting up log file, do I have permission to write to that location?."))
        .apply().expect("Error setting up logging. If log file is enabled, check write permissions.");

    // Create database connection pool
    let manager = Manager::new(CONFIG.db_address(), deadpool_diesel::Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();
    info!("Successfully connected to the database!");

    // TODO - Run pending DB migrations here

    // Make application state (atm just a struct containing DB connection pool)
    let state = AppState { pool: pool.clone() };

    // Setup tasks here
    tokio::spawn(tasks::sync_task::sync(pool.clone()));

    // Setup Axum JSON API
    let socket_addr = CONFIG.socket_address();
    info!("Starting API service!");
    let app = app_router().with_state(state);
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };
    
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {log::info!("Received CTRL+C signal, shutting down...")},
        _ = terminate => {log::info!("Received shutdown signal, shutting down...")},
    }
}

// Function to initialize tracing for logging
// fn init_tracing() {
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::EnvFilter::try_from_default_env()
//                 .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
//         )
//         .with(tracing_subscriber::fmt::layer())
//         .init();
// }

// Function to run database migrations
// async fn run_migrations(pool: &Pool) {
//     let conn = pool.get().await.unwrap();
//     conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
//         .await
//         .unwrap()
//         .unwrap();
// }