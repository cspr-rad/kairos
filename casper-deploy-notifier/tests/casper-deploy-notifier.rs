#[cfg(test)]
mod tests {
    use casper_deploy_notifier::*;
    use tokio::sync::mpsc;

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

    #[tokio::test]
    async fn test_run_without_connection() {
        let (tx, _rx) = mpsc::channel(1);
        let mut notifier = DeployNotifier::default();
        let result = notifier.run(tx).await;

        assert!(matches!(result, Err(SseError::NotConnected)));
    }

    #[tokio::test]
    async fn test_handling_node_shutdown() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body(concat!(
                "data: {\"ApiVersion\": \"1.5.6\"}\n\n",
                "data: \"Shutdown\"\n\n"
            ))
            .create_async()
            .await;

        let mut notifier = DeployNotifier::new(&server.url());
        notifier.connect().await.unwrap();

        let (tx, _rx) = mpsc::channel(1);
        let result = notifier.run(tx).await;
        assert!(matches!(result, Err(SseError::NodeShutdown)));
    }

    #[tokio::test]
    async fn test_run_stream_exhausted() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body("data: {\"ApiVersion\": \"1.5.6\"}\n\n")
            .create_async()
            .await;

        let mut notifier = DeployNotifier::new(&server.url());
        notifier.connect().await.unwrap();

        let (tx, _rx) = mpsc::channel(1);
        let result = notifier.run(tx).await;
        assert!(matches!(result, Err(SseError::StreamExhausted)));
    }

    #[tokio::test]
    async fn test_unexpected_handshake() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body(concat!(
                "data: {\"ApiVersion\": \"1.5.6\"}\n\n",
                "data: {\"ApiVersion\": \"1.5.6\"}\n\n"
            ))
            .create_async()
            .await;

        let mut notifier = DeployNotifier::new(&server.url());
        notifier.connect().await.unwrap();

        let (tx, _rx) = mpsc::channel(1);
        let result = notifier.run(tx).await;
        assert!(matches!(result, Err(SseError::UnexpectedHandshake)));
    }
}
