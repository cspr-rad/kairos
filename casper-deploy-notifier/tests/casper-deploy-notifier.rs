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
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_header("content-type", "text/event-stream")
            .with_body(concat!(
                "data: {\"ApiVersion\": \"1.5.6\"}\n\n",
                "data: {\"DeployProcessed\": {\"deploy_hash\":\"5322ee2dd5a8a812063dcb6c506f8cd5c2a0dce1c1d0320498a42c12f9280ede\",\"account\":\"016acb4cfa2ec31ea67ca53c1f93c77dba6740c463968ac550466723dc2cbaa421\",\"timestamp\":\"2024-07-11T10:10:57.073Z\",\"ttl\":\"30m\",\"dependencies\": [],\"block_hash\":\"90145cba9e25adf02f7acbab7e85b8a46e15ad86f28c5717dd4d548c5f14c908\",\"execution_result\": {\"Success\": {\"effect\": {\"operations\": [],\"transforms\": [{\"key\":\"account-hash-6174cf2e6f8fed1715c9a3bace9c50bfe572eecb763b0ed3f644532616452008\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d63c44078a1931b5dc4b80a7a0ec586164fd0470ce9f8b23f6d93b9e86c5944d\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"hash-4475016098705466254edd18d267a9dad43e341d4dafadb507d0fe3cf2d4a74b\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"balance-f1831c6b4d8535b124509ef947be9c8be9d99731b3ef20ef7ac1300e24efdc5f\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\":\"Identity\"},{\"key\":\"balance-f1831c6b4d8535b124509ef947be9c8be9d99731b3ef20ef7ac1300e24efdc5f\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"U512\",\"bytes\":\"0400ca9a3b\",\"parsed\":\"1000000000\"}}},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\": {\"AddUInt512\":\"4000000000\"}},{\"key\":\"hash-bf12dc357758d68706e5e1cd83a7208672b064f3feb63e263ff0c9c6929e5d17\",\"transform\":\"Identity\"},{\"key\":\"hash-a33e1ca85577e4593312338d0957c38d07820f4117ed081e4bc20df375bdb664\",\"transform\":\"Identity\"},{\"key\":\"hash-1662c7c8c0314e30f02f607aba5c2a7d7276cee81d1ad2757fa949dc21df48ae\",\"transform\":\"Identity\"},{\"key\":\"dictionary-8c03a95f560181cfe0700d03a0f319b01b96989521f32576731c73e08b01afd2\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"Any\",\"bytes\":\"28000000240000000100b664d0d775fcade88220684e8aed96a73310dcfc00c39f690e6296322c4d0e6201ea0e0320000000b6bf456cc4b13f1f617a96f6e096efcf5d4df167691fd86410ce473f4a55eab44000000039323962393833336636633366343638326535623339316234376536613531343532386231356661323435666334353130636462636262333534396638393661\",\"parsed\":null}}},{\"key\":\"uref-ac7408079dd7e5eb9e657b5e1188a1c6acb3d5e1654bb85054ec6d96732706da-000\",\"transform\":\"Identity\"},{\"key\":\"dictionary-ffb51ad54a5f1bc29e46e46915d2e2111682e659f268b738bf26c929f7720dd5\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"Any\",\"bytes\":\"2900000025000000100000006576656e745f5375626d697373696f6e10584014f3efd86369d7152c724113b8f20e0320000000bafb7dad56d5dc280fdaead4ba32eddaa203dad65d7ae01e5890e55923a0e3440400000038323937\",\"parsed\":null}}},{\"key\":\"uref-ac7408079dd7e5eb9e657b5e1188a1c6acb3d5e1654bb85054ec6d96732706da-000\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"U32\",\"bytes\":\"6a200000\",\"parsed\":8298}}},{\"key\":\"deploy-5322ee2dd5a8a812063dcb6c506f8cd5c2a0dce1c1d0320498a42c12f9280ede\",\"transform\": {\"WriteDeployInfo\": {\"deploy_hash\":\"5322ee2dd5a8a812063dcb6c506f8cd5c2a0dce1c1d0320498a42c12f9280ede\",\"transfers\": [],\"from\":\"account-hash-b664d0d775fcade88220684e8aed96a73310dcfc00c39f690e6296322c4d0e62\",\"source\":\"uref-f1831c6b4d8535b124509ef947be9c8be9d99731b3ef20ef7ac1300e24efdc5f-007\",\"gas\":\"410861701\"}}},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d63c44078a1931b5dc4b80a7a0ec586164fd0470ce9f8b23f6d93b9e86c5944d\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"account-hash-b664d0d775fcade88220684e8aed96a73310dcfc00c39f690e6296322c4d0e62\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"hash-4475016098705466254edd18d267a9dad43e341d4dafadb507d0fe3cf2d4a74b\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\":\"Identity\"},{\"key\":\"balance-f1831c6b4d8535b124509ef947be9c8be9d99731b3ef20ef7ac1300e24efdc5f\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"U512\",\"bytes\":\"043ce9a01a\",\"parsed\":\"446753084\"}}},{\"key\":\"balance-f1831c6b4d8535b124509ef947be9c8be9d99731b3ef20ef7ac1300e24efdc5f\",\"transform\": {\"AddUInt512\":\"3553246916\"}},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"hash-4475016098705466254edd18d267a9dad43e341d4dafadb507d0fe3cf2d4a74b\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\":\"Identity\"},{\"key\":\"balance-8078460a109aac95aaa8872f361ec6d5869608f4bea1b3dc3551541faf0c2ffc\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"U512\",\"bytes\":\"00\",\"parsed\":\"0\"}}},{\"key\":\"balance-8078460a109aac95aaa8872f361ec6d5869608f4bea1b3dc3551541faf0c2ffc\",\"transform\": {\"AddUInt512\":\"446753084\"}}]},\"transfers\": [],\"cost\":\"410861701\"}}}}\n\n",
                "data: {\"DeployProcessed\": {\"deploy_hash\":\"4b4cf10f2ebb9df0e754e1849a4977f57bba0a20a8f58c73b970b69f26153c64\",\"account\":\"01f03bbc42a3d5901c7232987ba84ab2c6d210973a0cfe742284dcb1d8b4cbe1c3\",\"timestamp\":\"2024-07-11T10:11:27.512Z\",\"ttl\":\"30m\",\"dependencies\": [],\"block_hash\":\"926f7c831d6313b1359a64ae01845e678da337381f228f1be7230476e594899e\",\"execution_result\": {\"Success\": {\"effect\": {\"operations\": [],\"transforms\": [{\"key\":\"account-hash-6174cf2e6f8fed1715c9a3bace9c50bfe572eecb763b0ed3f644532616452008\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d63c44078a1931b5dc4b80a7a0ec586164fd0470ce9f8b23f6d93b9e86c5944d\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"hash-4475016098705466254edd18d267a9dad43e341d4dafadb507d0fe3cf2d4a74b\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"balance-c81c49b0c44769d1cb841e83452cd08d133b9f8d357a3cc561d0923130047db0\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\":\"Identity\"},{\"key\":\"balance-c81c49b0c44769d1cb841e83452cd08d133b9f8d357a3cc561d0923130047db0\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"U512\",\"bytes\":\"06a5d4874e1204\",\"parsed\":\"4476673447077\"}}},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\": {\"AddUInt512\":\"5000000000\"}},{\"key\":\"hash-bf12dc357758d68706e5e1cd83a7208672b064f3feb63e263ff0c9c6929e5d17\",\"transform\":\"Identity\"},{\"key\":\"hash-a33e1ca85577e4593312338d0957c38d07820f4117ed081e4bc20df375bdb664\",\"transform\":\"Identity\"},{\"key\":\"hash-1662c7c8c0314e30f02f607aba5c2a7d7276cee81d1ad2757fa949dc21df48ae\",\"transform\":\"Identity\"},{\"key\":\"dictionary-efede2879aa85603991041b248a2809592abdb842975177a1207f45b2631e740\",\"transform\":\"Identity\"},{\"key\":\"dictionary-8c03a95f560181cfe0700d03a0f319b01b96989521f32576731c73e08b01afd2\",\"transform\":\"Identity\"},{\"key\":\"dictionary-8c03a95f560181cfe0700d03a0f319b01b96989521f32576731c73e08b01afd2\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"Any\",\"bytes\":\"0500000001000000000e0320000000b6bf456cc4b13f1f617a96f6e096efcf5d4df167691fd86410ce473f4a55eab44000000039323962393833336636633366343638326535623339316234376536613531343532386231356661323435666334353130636462636262333534396638393661\",\"parsed\":null}}},{\"key\":\"dictionary-cfeea884d140a02d69f60658e79c52d273acae55f87a03ea05d97e948e0c2036\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"Any\",\"bytes\":\"060000000200000001ea0e0320000000b6bf456cc4b13f1f617a96f6e096efcf5d4df167691fd86410ce473f4a55eab44000000035653462616434356536623364383261306133343734653362623565373061653663663030633030623437636466303466323936616234383463346365643166\",\"parsed\":null}}},{\"key\":\"uref-ac7408079dd7e5eb9e657b5e1188a1c6acb3d5e1654bb85054ec6d96732706da-000\",\"transform\":\"Identity\"},{\"key\":\"dictionary-2e89d89ee7ce94674b959e57519711dd3525d88f83c12cc04c1a292e8d157b6a\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"Any\",\"bytes\":\"2b00000027000000120000006576656e745f436f6e6669726d6174696f6e10584014f3efd86369d7152c724113b8f20e0320000000bafb7dad56d5dc280fdaead4ba32eddaa203dad65d7ae01e5890e55923a0e3440400000038323938\",\"parsed\":null}}},{\"key\":\"uref-ac7408079dd7e5eb9e657b5e1188a1c6acb3d5e1654bb85054ec6d96732706da-000\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"U32\",\"bytes\":\"6b200000\",\"parsed\":8299}}},{\"key\":\"deploy-4b4cf10f2ebb9df0e754e1849a4977f57bba0a20a8f58c73b970b69f26153c64\",\"transform\": {\"WriteDeployInfo\": {\"deploy_hash\":\"4b4cf10f2ebb9df0e754e1849a4977f57bba0a20a8f58c73b970b69f26153c64\",\"transfers\": [],\"from\":\"account-hash-0a12fef621d43e5dfa0845065371adc816a92ad40e35b0c311de9680445eabbd\",\"source\":\"uref-c81c49b0c44769d1cb841e83452cd08d133b9f8d357a3cc561d0923130047db0-007\",\"gas\":\"524513374\"}}},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"hash-d63c44078a1931b5dc4b80a7a0ec586164fd0470ce9f8b23f6d93b9e86c5944d\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\":\"Identity\"},{\"key\":\"hash-d2469afeb99130f0be7c9ce230a84149e6d756e306ef8cf5b8a49d5182e41676\",\"transform\":\"Identity\"},{\"key\":\"account-hash-0a12fef621d43e5dfa0845065371adc816a92ad40e35b0c311de9680445eabbd\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"hash-4475016098705466254edd18d267a9dad43e341d4dafadb507d0fe3cf2d4a74b\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\":\"Identity\"},{\"key\":\"balance-c81c49b0c44769d1cb841e83452cd08d133b9f8d357a3cc561d0923130047db0\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"U512\",\"bytes\":\"041158ee21\",\"parsed\":\"569268241\"}}},{\"key\":\"balance-c81c49b0c44769d1cb841e83452cd08d133b9f8d357a3cc561d0923130047db0\",\"transform\": {\"AddUInt512\":\"4430731759\"}},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"hash-4475016098705466254edd18d267a9dad43e341d4dafadb507d0fe3cf2d4a74b\",\"transform\":\"Identity\"},{\"key\":\"hash-7cc1b1db4e08bbfe7bacf8e1ad828a5d9bcccbb33e55d322808c3a88da53213a\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\":\"Identity\"},{\"key\":\"balance-065a7ba7eebf16ef1f47b7b1f2bb3c8c5019b7762b81cafcbc1bf9c15b553126\",\"transform\":\"Identity\"},{\"key\":\"balance-fe327f9815a1d016e1143db85e25a86341883949fd75ac1c1e7408a26c5b62ef\",\"transform\": {\"WriteCLValue\": {\"cl_type\":\"U512\",\"bytes\":\"00\",\"parsed\":\"0\"}}},{\"key\":\"balance-065a7ba7eebf16ef1f47b7b1f2bb3c8c5019b7762b81cafcbc1bf9c15b553126\",\"transform\": {\"AddUInt512\":\"569268241\"}}]},\"transfers\": [],\"cost\":\"524513374\"}}}}\n\n",
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
                "5322ee2dd5a8a812063dcb6c506f8cd5c2a0dce1c1d0320498a42c12f9280ede"
            );
            assert_eq!(
                notification.public_key,
                "016acb4cfa2ec31ea67ca53c1f93c77dba6740c463968ac550466723dc2cbaa421"
            );
            assert_eq!(notification.success, true);
        } else {
            panic!("Expected a notification, but none was received");
        }
        if let Some(notification) = rx.recv().await {
            assert_eq!(
                notification.deploy_hash,
                "4b4cf10f2ebb9df0e754e1849a4977f57bba0a20a8f58c73b970b69f26153c64"
            );
            assert_eq!(
                notification.public_key,
                "01f03bbc42a3d5901c7232987ba84ab2c6d210973a0cfe742284dcb1d8b4cbe1c3"
            );
            assert_eq!(notification.success, true);
        } else {
            panic!("Expected a second notification, but none was received");
        }
    }
}
