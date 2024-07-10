use backoff::{retry, Error, ExponentialBackoff};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::Url;
use std::env;
use std::fs::{self, File};
use std::io;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command};
use tempfile::{tempdir, TempDir};
use tracing::{info, warn};

#[derive(Clone)]
pub struct PgConnection {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
}

impl From<PgConnection> for Url {
    fn from(
        PgConnection {
            host,
            port,
            username,
            database,
        }: PgConnection,
    ) -> Url {
        const FRAGMENT: &AsciiSet = &CONTROLS.add(b'/');
        let host_encoded = utf8_percent_encode(&host, FRAGMENT).to_string();
        Url::parse(&format!(
            "postgresql://{}@{}:{}/{}",
            username, host_encoded, port, database
        ))
        .unwrap()
    }
}

pub struct PostgresDB {
    pub connection: PgConnection,
    pub working_dir: TempDir,
    process_handle: Child,
}

impl PostgresDB {
    pub fn run(migrations_dir_env: &str) -> Result<PostgresDB, io::Error> {
        let migrations = match env::var(migrations_dir_env) {
            Ok(migrations_dir) => fs::read_dir(migrations_dir)?
                .map(|res| res.map(|p| p.path()))
                .collect::<Result<Vec<_>, io::Error>>()?,
            Err(_) => {
                warn!("The environment variable defining the database migrations directory `{}` is not set", migrations_dir_env);
                vec![]
            }
        };
        let working_dir = tempdir()?;
        let database_dir = working_dir.path().join("data");
        let socket = working_dir.path().join("socket");
        let port = TcpListener::bind("0.0.0.0:0")?.local_addr()?.port();
        let postgres_log =
            File::create(working_dir.path().join("postgres.log")).expect("failed to open log");

        std::fs::create_dir(&database_dir)?;
        std::fs::create_dir(&socket)?;

        Command::new("initdb")
            .args([database_dir.to_str().unwrap()])
            .output()
            .expect("Initializing database failed");

        info!("Starting postgres...");

        let postgres_handle = Command::new("postgres")
            .args([
                "-D",
                database_dir.to_str().unwrap(),
                "-k",
                socket.to_str().unwrap(),
                "-p",
                &port.to_string(),
            ])
            .stderr(postgres_log)
            .spawn()
            .expect("Failed to start postgres.");

        info!(
            "Waiting for postgres to open the socket '{}' ...",
            socket.to_str().unwrap()
        );

        wait_for_socket(&socket.join(format!(".s.PGSQL.{}", port)))
            .map_err(|err| io::Error::new(io::ErrorKind::NotFound, err.to_string()))?;

        info!("Postgres socket open");

        let pg_user = "helloworld";
        let pg_database = "helloworld";

        info!("Creating user...");

        Command::new("createuser")
            .args([
                "--createdb",
                "--superuser",
                pg_user,
                "-h",
                socket.to_str().unwrap(),
                "-p",
                &port.to_string(),
            ])
            .output()
            .expect("Creating user failed.");

        info!("Creating database for user...");

        Command::new("createdb")
            .args([
                "-h",
                socket.to_str().unwrap(),
                "-p",
                &port.to_string(),
                "-U",
                pg_user,
                "-O",
                pg_user,
                pg_database,
            ])
            .output()
            .expect("Creating database failed");

        info!("Running the migrations");
        for migration in migrations {
            info!("Running migration {}", migration.to_str().unwrap());
            Command::new("psql")
                .args([
                    "-h",
                    socket.to_str().unwrap(),
                    "-p",
                    &port.to_string(),
                    "-U",
                    pg_user,
                    pg_database,
                    "-f",
                    migration.to_str().unwrap(),
                ])
                .output()
                .expect("Failed to run migration");
        }

        let connection = PgConnection {
            host: socket.to_str().unwrap().to_string(),
            port,
            username: pg_user.to_string(),
            database: pg_database.to_string(),
        };

        Ok(PostgresDB {
            connection,
            working_dir,
            process_handle: postgres_handle,
        })
    }
}

impl Drop for PostgresDB {
    fn drop(&mut self) {
        let _ = self.process_handle.kill();
    }
}

fn wait_for_socket(path: &Path) -> Result<(), Error<String>> {
    retry(ExponentialBackoff::default(), || {
        if fs::metadata(path).is_ok() {
            Ok(())
        } else {
            Err(Error::transient(format!(
                "Unable to find socket {}",
                path.to_str().unwrap()
            )))
        }
    })
}
