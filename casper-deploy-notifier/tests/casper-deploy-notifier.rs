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

    #[tokio::test]
    async fn test_parsing_real_deploy_event() {
        tracing_subscriber::fmt::init();
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body(concat!(
                "data: {\"ApiVersion\": \"1.5.6\"}\n\n",
                "data:{\"TransactionProcessed\":{\"transaction_hash\":{\"Deploy\":\"6db5b8322d881b8a8d5a483af52e1abc770b608ef60db53e0e84ba9838df320c\"},\"initiator_addr\":{\"PublicKey\":\"0138329930033bca4773a6623574ad7870ee39c554f153f15609e200e50049a7de\"},\"timestamp\":\"2024-09-17T10:07:02.572Z\",\"ttl\":\"5m\",\"block_hash\":\"d9d2e9da47fc154731991169f330e441e469b87fcc2e16f4160f239408a0c634\",\"execution_result\":{\"Version2\":{\"initiator\":{\"PublicKey\":\"0138329930033bca4773a6623574ad7870ee39c554f153f15609e200e50049a7de\"},\"error_message\":null,\"limit\":\"10000\",\"consumed\":\"10000\",\"cost\":\"10000\",\"payment\":[],\"transfers\":[{\"Version2\":{\"transaction_hash\":{\"Deploy\":\"6db5b8322d881b8a8d5a483af52e1abc770b608ef60db53e0e84ba9838df320c\"},\"from\":{\"AccountHash\":\"account-hash-aab0da01340446cee477f28410f8af5d6e0f3a88fb26c0cafb8d1625f5cc9c10\"},\"to\":\"account-hash-5a9eb1f7da515d9fa2f0b74e18ec84cccf90f146269d538073416dff432a3c77\",\"source\":\"uref-c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2b-007\",\"target\":\"uref-157155ab70b6da0900efb492dcebd1293d11c16df81943ca86800eaea44e3e0f-004\",\"amount\":\"2500000000\",\"gas\":\"0\",\"id\":2}}],\"size_estimate\":368,\"effects\":[{\"key\":\"balance-hold-01c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":{\"Write\":{\"CLValue\":{\"cl_type\":\"U512\",\"bytes\":\"021027\",\"parsed\":\"10000\"}}}},{\"key\":\"account-hash-aab0da01340446cee477f28410f8af5d6e0f3a88fb26c0cafb8d1625f5cc9c10\",\"kind\":\"Identity\"},{\"key\":\"balance-hold-00c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":\"Identity\"},{\"key\":\"balance-hold-01c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":\"Identity\"},{\"key\":\"balance-c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2b\",\"kind\":{\"Write\":{\"CLValue\":{\"cl_type\":\"U512\",\"bytes\":\"0e001cf4ab075bc138938d44c64d31\",\"parsed\":\"999999999999999999999990000000000\"}}}},{\"key\":\"balance-157155ab70b6da0900efb492dcebd1293d11c16df81943ca86800eaea44e3e0f\",\"kind\":{\"AddUInt512\":\"2500000000\"}},{\"key\":\"balance-hold-01c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":{\"Prune\":\"balance-hold-01c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\"}},{\"key\":\"balance-hold-00c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":{\"Write\":{\"CLValue\":{\"cl_type\":\"U512\",\"bytes\":\"02409c\",\"parsed\":\"40000\"}}}},{\"key\":\"entity-system-d0c844216a182e2d02788e06c4701d91dc1a53fba666bad6b8cd3d4689333869\",\"kind\":\"Identity\"},{\"key\":\"entity-system-de3db671c5fa4b802d9b0db9bc30c44f0e8fee842ace5d80a925eab1b4b164e2\",\"kind\":\"Identity\"},{\"key\":\"entity-system-84a93da99fd7605f2ec2d14274570ad84ddfc4a0ebd913cbabb7fb6dbcc339f3\",\"kind\":\"Identity\"},{\"key\":\"bid-addr-0186c8312d6cf30e420287d6432173db950937140bae24a0c18ea83e3e2c5e17fa\",\"kind\":\"Identity\"},{\"key\":\"bid-addr-0486c8312d6cf30e420287d6432173db950937140bae24a0c18ea83e3e2c5e17fa0200000000000000\",\"kind\":\"Identity\"},{\"key\":\"bid-addr-0486c8312d6cf30e420287d6432173db950937140bae24a0c18ea83e3e2c5e17fa0200000000000000\",\"kind\":{\"Write\":{\"BidKind\":{\"Credit\":{\"validator_public_key\":\"01706f36a2ebfccea720b49a6424c196cf0bb7aa929f39842975865848b87773ef\",\"era_id\":2,\"amount\":\"10000\"}}}}}]}},\"messages\":[]}}\n\n",
                "data:{\"TransactionProcessed\":{\"transaction_hash\":{\"Deploy\":\"dceae78c7ffce8b9d45837adcf6c4cf902b51ee497703145ff4e4ad756d98708\"},\"initiator_addr\":{\"PublicKey\":\"0138329930033bca4773a6623574ad7870ee39c554f153f15609e200e50049a7de\"},\"timestamp\":\"2024-09-17T10:07:02.793Z\",\"ttl\":\"5m\",\"block_hash\":\"d9d2e9da47fc154731991169f330e441e469b87fcc2e16f4160f239408a0c634\",\"execution_result\":{\"Version2\":{\"initiator\":{\"PublicKey\":\"0138329930033bca4773a6623574ad7870ee39c554f153f15609e200e50049a7de\"},\"error_message\":null,\"limit\":\"10000\",\"consumed\":\"10000\",\"cost\":\"10000\",\"payment\":[],\"transfers\":[{\"Version2\":{\"transaction_hash\":{\"Deploy\":\"dceae78c7ffce8b9d45837adcf6c4cf902b51ee497703145ff4e4ad756d98708\"},\"from\":{\"AccountHash\":\"account-hash-aab0da01340446cee477f28410f8af5d6e0f3a88fb26c0cafb8d1625f5cc9c10\"},\"to\":\"account-hash-5a9eb1f7da515d9fa2f0b74e18ec84cccf90f146269d538073416dff432a3c77\",\"source\":\"uref-c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2b-007\",\"target\":\"uref-157155ab70b6da0900efb492dcebd1293d11c16df81943ca86800eaea44e3e0f-004\",\"amount\":\"2500000000\",\"gas\":\"0\",\"id\":4}}],\"size_estimate\":368,\"effects\":[{\"key\":\"balance-hold-01c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":{\"Write\":{\"CLValue\":{\"cl_type\":\"U512\",\"bytes\":\"021027\",\"parsed\":\"10000\"}}}},{\"key\":\"account-hash-aab0da01340446cee477f28410f8af5d6e0f3a88fb26c0cafb8d1625f5cc9c10\",\"kind\":\"Identity\"},{\"key\":\"balance-hold-00c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":\"Identity\"},{\"key\":\"balance-hold-01c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":\"Identity\"},{\"key\":\"balance-c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2b\",\"kind\":{\"Write\":{\"CLValue\":{\"cl_type\":\"U512\",\"bytes\":\"0e0023f116075bc138938d44c64d31\",\"parsed\":\"999999999999999999999987500000000\"}}}},{\"key\":\"balance-157155ab70b6da0900efb492dcebd1293d11c16df81943ca86800eaea44e3e0f\",\"kind\":{\"AddUInt512\":\"2500000000\"}},{\"key\":\"balance-hold-01c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":{\"Prune\":\"balance-hold-01c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\"}},{\"key\":\"balance-hold-00c006b3b3ccafd537b647a85f8abe2b7e346232e0b2b9343659cb924072b23d2bab2e73ff91010000\",\"kind\":{\"Write\":{\"CLValue\":{\"cl_type\":\"U512\",\"bytes\":\"0250c3\",\"parsed\":\"50000\"}}}},{\"key\":\"entity-system-d0c844216a182e2d02788e06c4701d91dc1a53fba666bad6b8cd3d4689333869\",\"kind\":\"Identity\"},{\"key\":\"entity-system-de3db671c5fa4b802d9b0db9bc30c44f0e8fee842ace5d80a925eab1b4b164e2\",\"kind\":\"Identity\"},{\"key\":\"entity-system-84a93da99fd7605f2ec2d14274570ad84ddfc4a0ebd913cbabb7fb6dbcc339f3\",\"kind\":\"Identity\"},{\"key\":\"bid-addr-0186c8312d6cf30e420287d6432173db950937140bae24a0c18ea83e3e2c5e17fa\",\"kind\":\"Identity\"},{\"key\":\"bid-addr-0486c8312d6cf30e420287d6432173db950937140bae24a0c18ea83e3e2c5e17fa0200000000000000\",\"kind\":\"Identity\"},{\"key\":\"bid-addr-0486c8312d6cf30e420287d6432173db950937140bae24a0c18ea83e3e2c5e17fa0200000000000000\",\"kind\":{\"Write\":{\"BidKind\":{\"Credit\":{\"validator_public_key\":\"01706f36a2ebfccea720b49a6424c196cf0bb7aa929f39842975865848b87773ef\",\"era_id\":2,\"amount\":\"10000\"}}}}}]}},\"messages\":[]}}\n\n",
                "data: \"Shutdown\"\n\n"
            ))
            .create_async()
            .await;

        let mut notifier = DeployNotifier::new(&server.url());
        notifier.connect().await.unwrap();

        let (tx, mut rx) = mpsc::channel(100);
        let result = notifier.run(tx).await;
        assert!(matches!(result, Err(SseError::NodeShutdown)));

        // Check if the notifications were received properly before shutdown
        if let Some(notification) = rx.recv().await {
            assert_eq!(
                notification.deploy_hash,
                "6db5b8322d881b8a8d5a483af52e1abc770b608ef60db53e0e84ba9838df320c"
            );
            assert_eq!(
                notification.public_key,
                "aab0da01340446cee477f28410f8af5d6e0f3a88fb26c0cafb8d1625f5cc9c10"
            );
            assert!(notification.success);
        } else {
            panic!("Expected a notification, but none was received");
        }
        if let Some(notification) = rx.recv().await {
            assert_eq!(
                notification.deploy_hash,
                "dceae78c7ffce8b9d45837adcf6c4cf902b51ee497703145ff4e4ad756d98708"
            );
            assert_eq!(
                notification.public_key,
                "aab0da01340446cee477f28410f8af5d6e0f3a88fb26c0cafb8d1625f5cc9c10"
            );
            assert!(notification.success);
        } else {
            panic!("Expected a second notification, but none was received");
        }
    }

    #[tokio::test]
    async fn test_ignoring_other_events() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body(concat!(
                "data: {\"ApiVersion\": \"1.5.6\"}\n\n",
                "data: {\"Foo\": \"Bar\"}\n\n",
                "data: \"Shutdown\"\n\n"
            ))
            .create_async()
            .await;

        let mut notifier = DeployNotifier::new(&server.url());
        notifier.connect().await.unwrap();

        let (tx, mut rx) = mpsc::channel(1);
        let result = notifier.run(tx).await;
        assert!(matches!(result, Err(SseError::NodeShutdown)));

        // There was no deploy event, so no notification expected.
        let maybe_notification = rx.recv().await;
        assert!(maybe_notification.is_none());
    }
}
