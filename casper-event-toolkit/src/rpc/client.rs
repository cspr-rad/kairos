use casper_client::types::StoredValue;
use casper_hashing::Digest;
use casper_types::{CLValue, HashAddr, URef};

use crate::error::ToolkitError;
use crate::rpc::id_generator::JsonRpcIdGenerator;

pub const DEFAULT_MAINNET_RPC_ENDPOINT: &str = "https://mainnet.casper-node.xyz/rpc";
pub const DEFAULT_TESTNET_RPC_ENDPOINT: &str = "https://testnet.casper-node.xyz/rpc";

pub struct CasperClient {
    rpc_endpoint: String,
    id_generator: JsonRpcIdGenerator,
}

impl CasperClient {
    pub fn new(rpc_endpoint: &str) -> Self {
        Self {
            rpc_endpoint: rpc_endpoint.to_string(),
            id_generator: JsonRpcIdGenerator::default(),
        }
    }

    pub fn default_mainnet() -> Self {
        Self::new(DEFAULT_MAINNET_RPC_ENDPOINT)
    }

    pub fn default_testnet() -> Self {
        Self::new(DEFAULT_TESTNET_RPC_ENDPOINT)
    }

    // Fetch latest state root hash.
    pub(crate) async fn get_state_root_hash(&self) -> Result<Digest, ToolkitError> {
        // No block given means the latest available.
        let block_identifier = None;

        // Common parameters.
        let rpc_id = self.id_generator.next_id().into();
        let verbosity = casper_client::Verbosity::Low;

        let response = casper_client::get_state_root_hash(
            rpc_id,
            &self.rpc_endpoint,
            verbosity,
            block_identifier,
        )
        .await?;

        let state_root_hash = match response.result.state_root_hash {
            Some(v) => Ok(v),
            None => Err(ToolkitError::UnexpectedError {
                context: "empty state root hash".into(),
            }),
        }?;

        Ok(state_root_hash)
    }

    async fn query_global_state(
        &self,
        state_root_hash: Digest,
        key: casper_types::Key,
        path: Vec<String>,
    ) -> Result<StoredValue, ToolkitError> {
        // Wrap state root hash.
        let global_state_identifier =
            casper_client::rpcs::GlobalStateIdentifier::StateRootHash(state_root_hash);

        // Common parameters.
        let rpc_id = self.id_generator.next_id().into();
        let verbosity = casper_client::Verbosity::Low;

        let response = casper_client::query_global_state(
            rpc_id,
            &self.rpc_endpoint,
            verbosity,
            global_state_identifier,
            key,
            path,
        )
        .await?;
        let stored_value = response.result.stored_value;

        Ok(stored_value)
    }

    pub(crate) async fn get_contract_named_keys(
        &self,
        contract_hash: HashAddr,
    ) -> Result<casper_types::contracts::NamedKeys, ToolkitError> {
        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await?;

        // Contract is stored directly at given hash.
        let key = casper_types::Key::Hash(contract_hash);
        let path = vec![];

        let stored_value = self.query_global_state(state_root_hash, key, path).await?;
        let contract = match stored_value {
            casper_client::types::StoredValue::Contract(v) => Ok(v),
            _ => Err(ToolkitError::UnexpectedStoredValueType {
                expected_type: "contract",
            }),
        }?;

        // Casper client use different type of named keys, so we have to additionally parse it.
        let contract = crate::rpc::utils::extract_named_keys(contract);

        Ok(contract)
    }

    pub(crate) async fn get_stored_clvalue(
        &self,
        uref: &casper_types::URef,
    ) -> Result<CLValue, ToolkitError> {
        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await?;

        // Build uref key.
        let key = casper_types::Key::URef(*uref);
        let path = vec![];

        let stored_value = self.query_global_state(state_root_hash, key, path).await?;
        let clvalue = match stored_value {
            casper_client::types::StoredValue::CLValue(v) => Ok(v),
            _ => Err(ToolkitError::UnexpectedStoredValueType {
                expected_type: "clvalue",
            }),
        }?;

        Ok(clvalue)
    }

    pub(crate) async fn get_stored_clvalue_from_dict(
        &self,
        dictionary_seed_uref: &URef,
        dictionary_item_key: &str,
    ) -> Result<CLValue, ToolkitError> {
        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await?;

        // Build dictionary item identifier.
        let dictionary_item_key = dictionary_item_key.to_string();
        let dictionary_item_identifier =
            casper_client::rpcs::DictionaryItemIdentifier::new_from_seed_uref(
                *dictionary_seed_uref,
                dictionary_item_key,
            );

        // Common parameters.
        let rpc_id = self.id_generator.next_id().into();
        let verbosity = casper_client::Verbosity::Low;

        let response = casper_client::get_dictionary_item(
            rpc_id,
            &self.rpc_endpoint,
            verbosity,
            state_root_hash.into(),
            dictionary_item_identifier,
        )
        .await?;
        let stored_value = response.result.stored_value;
        let clvalue = match stored_value {
            casper_client::types::StoredValue::CLValue(v) => Ok(v),
            _ => Err(ToolkitError::UnexpectedStoredValueType {
                expected_type: "clvalue",
            }),
        }?;

        Ok(clvalue)
    }

    pub(crate) async fn get_deploy_result(
        &self,
        deploy_hash: casper_client::types::DeployHash,
    ) -> Result<casper_types::ExecutionResult, ToolkitError> {
        // Approvals originally received by the node are okay.
        let finalized_approvals = false;

        // Common parameters.
        let rpc_id = self.id_generator.next_id().into();
        let verbosity = casper_client::Verbosity::Low;

        let response = casper_client::get_deploy(
            rpc_id,
            &self.rpc_endpoint,
            verbosity,
            deploy_hash,
            finalized_approvals,
        )
        .await?;
        let mut execution_results = response.result.execution_results;
        assert_eq!(execution_results.len(), 1); // TODO: Is it always the case?
        let execution_result = execution_results.remove(0);

        Ok(execution_result.result)
    }
}
