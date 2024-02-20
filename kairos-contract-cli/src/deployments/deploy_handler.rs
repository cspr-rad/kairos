use casper_client::{put_deploy, rpcs::results::PutDeployResult, JsonRpcId, Verbosity};
use casper_types::{crypto::SecretKey, Deploy, DeployBuilder, ExecutableDeployItem};

pub struct LayerOneDeployHandler {
    pub node_address: String,
    pub rpc_port: JsonRpcId,
    pub secret_key: SecretKey,
}

impl LayerOneDeployHandler {
    pub fn build_deploy(
        &self,
        chain_name: &str,
        session: ExecutableDeployItem,
        secret_key: &SecretKey,
        payment: u64,
    ) -> Deploy {
        DeployBuilder::new(chain_name, session)
            .with_standard_payment(payment)
            .with_secret_key(secret_key)
            .build()
            .unwrap()
    }
    pub async fn put_deploy(&self, deploy: Deploy) -> PutDeployResult {
        put_deploy(
            self.rpc_port.clone(),
            &self.node_address,
            Verbosity::Low,
            deploy,
        )
        .await
        .unwrap()
        .result
    }
}
