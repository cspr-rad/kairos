#[cfg(test)]
mod tests {
    use casper_deploy_notifier::*;

    #[tokio::test]
    async fn test_connection_failure() {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/").with_status(500);

        let mut notifier = DeployNotifier::new(&server.url());
        let result = notifier.connect().await;
        assert!(matches!(result, Err(SseError::ConnectionError(_))));
    }

    #[tokio::test]
    async fn test_connection_success() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body("data: {\"ApiVersion\": \"1.5.6\"}\n\n")
            .create_async()
            .await;

        let mut notifier = DeployNotifier::new(&server.url());
        let result = notifier.connect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_ended_before_handshake() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body("")
            .create_async()
            .await;

        let mut notifier = DeployNotifier::new(&server.url());
        let result = notifier.connect().await;
        assert!(matches!(result, Err(SseError::StreamExhausted)));
    }

    #[tokio::test]
    async fn test_invalid_handshake() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body("data: {\"Foo\": \"bar\"}\n\n")
            .create_async()
            .await;

        let mut notifier = DeployNotifier::new(&server.url());
        let result = notifier.connect().await;
        assert!(matches!(result, Err(SseError::InvalidHandshake)));
    }
}
