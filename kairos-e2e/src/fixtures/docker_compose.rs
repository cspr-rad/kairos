/// This code wouldn't exist if not for: https://github.com/shotover/docker-compose-runner

use std::io::ErrorKind;
use std::time::{self, Duration};
use tracing::trace;
use tokio::process::Command;
use std::process::Stdio;

/// Runs a command and returns the output as a string.
///
/// Both stderr and stdout are returned in the result.
///
/// # Arguments
/// * `command` - The system command to run
/// * `args` - An array of command line arguments for the command
pub async fn run_command(command: &str, args: &[&str]) -> Result<String, String> {
    trace!("executing {}", command);

    let output = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    return match output {
        Ok(o) => Ok(String::from_utf8(o.stdout).unwrap()),
        Err(e) => Err(e.to_string()),
    }
}

/// Launch and manage a docker compose instance
#[derive(Clone)]
pub struct DockerCompose {
    file_path: String,
    runtime_handle: Option<tokio::runtime::Handle>,
}

impl DockerCompose {
    /// Runs docker compose on the provided docker-compose.yaml file.
    /// Dropping the returned object will stop and destroy the launched docker compose services.
    pub async fn new(
        yaml_path: &str,
    ) -> Self {
        let runtime_handle = tokio::runtime::Handle::try_current()
            .map(|handle| handle.clone())
            .ok();

        match Command::new("docker")
            .arg("compose")
            .output()
            .await
            .map_err(|e| e.kind())
        {
            Err(ErrorKind::NotFound) => panic!("Could not find docker. Have you installed docker?"),
            Err(err) => panic!("error running docker {:?}", err),
            Ok(output) => {
                if !output.status.success() {
                    panic!("Could not find docker compose. Have you installed docker compose?");
                }
            }
        }

        // It is critical that clean_up is run before everything else as the internal `docker compose` commands act as validation
        // for the docker-compose.yaml file that we later manually parse with poor error handling
        DockerCompose::clean_up(yaml_path).await.unwrap();

        run_command("docker", &["compose", "-f", yaml_path, "up", "-d", "--remove-orphans"]).await.unwrap();

        DockerCompose::wait_for_health(yaml_path).await;

        DockerCompose {
            file_path: yaml_path.to_string(),
            runtime_handle,
        }
    }

    async fn wait_for_health(file_path: &str) {
        let timeout = Duration::from_secs(100);
        let start = time::Instant::now();

        let args = [
            "compose",
            "-f",
            file_path,
            "ps",
            "--services",
            "--filter",
            "status=running",
            "--format",
            "{{.Label \"com.docker.compose.service\"}}\t{{.State.Health.Status}}"
        ];

        while start.elapsed() < timeout {
            let output_str = run_command("docker", &args).await.unwrap();
    
            let all_healthy = output_str.lines().all(|line| {
                let parts: Vec<_> = line.split('\t').collect();
                parts.len() > 1 && parts[1] == "healthy"
            });
    
            if all_healthy {
                trace!("All services are healthy.");
                return;
            }
    
            std::thread::sleep(Duration::from_secs(5));  // Check every 5 seconds
        }
    
    }

    /// Cleans up docker compose by shutting down the running system and removing the images.
    ///
    /// # Arguments
    /// * `file_path` - The path to the docker-compose yaml file that was used to start docker.
    async fn clean_up(file_path: &str) -> Result<(), String> {
        trace!("bringing down docker compose {}", file_path);

        let _ = run_command("docker", &["compose", "-f", file_path, "down", "-v"]).await;       

        Ok(())
    }
}

impl Drop for DockerCompose {
    fn drop(&mut self) {
        if let Some(runtime_handle) = &self.runtime_handle {
            let file_path = self.file_path.clone();
            runtime_handle.spawn(async move {
                if let Err(err) = DockerCompose::clean_up(&file_path).await {
                    eprintln!("Failed to clean up DockerCompose instance: {}", err);
                }
            });
        } else {
            panic!("Failed to fetch Tokio runtime, DockerCompose fixture Drop trait broken.")
        }
    }
}