use casper_client::rpcs::results::QueryGlobalStateResult;
use casper_types::{CLValue, URef};

use crate::rpc::id::JsonRpcIdGenerator;

const DEFAULT_MAINNET_RPC: &str = "https://mainnet.casper-node.xyz/rpc";
const DEFAULT_TESTNET_RPC: &str = "https://testnet.casper-node.xyz/rpc";

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
        Self::new(DEFAULT_MAINNET_RPC)
    }

    pub fn default_testnet() -> Self {
        Self::new(DEFAULT_TESTNET_RPC)
    }

    // Fetch latest state root hash.
    pub async fn get_state_root_hash(&self) -> [u8; 32] {
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
        .await
        .unwrap();

        let state_root_hash = response.result.state_root_hash.unwrap();

        state_root_hash.into()
    }

    async fn query_global_state(
        &self,
        state_root_hash: &[u8; 32],
        key: casper_types::Key,
        path: Vec<String>,
    ) -> QueryGlobalStateResult {
        // Wrap state root hash.
        let global_state_identifier = casper_client::rpcs::GlobalStateIdentifier::StateRootHash(
            state_root_hash.clone().into(),
        );

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
        .await
        .unwrap();

        response.result
    }

    pub async fn get_contract_named_keys(
        &self,
        contract_hash: &str,
    ) -> casper_types::contracts::NamedKeys {
        // Build contract hash.
        let contract_hash_bytes = hex::decode(contract_hash).unwrap();
        let contract_hash_bytes: [u8; 32] = contract_hash_bytes.try_into().unwrap();
        let contract_hash = casper_types::ContractWasmHash::new(contract_hash_bytes);

        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await;

        // Contract is stored directly at given hash.
        let key = casper_types::Key::Hash(contract_hash.value());
        let path = vec![];

        let response = self.query_global_state(&state_root_hash, key, path).await;
        let contract = match response.stored_value {
            casper_client::types::StoredValue::Contract(v) => v,
            _ => panic!("Expected contract."),
        };

        // Casper client use different type of named keys, so we have to additionally parse it.
        let contract = crate::rpc::utils::extract_named_keys(contract);

        contract
    }

    pub async fn get_stored_clvalue(&self, uref: &casper_types::URef) -> CLValue {
        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await;

        // Build uref key.
        let key = casper_types::Key::URef(*uref);
        let path = vec![];

        let response = self.query_global_state(&state_root_hash, key, path).await;
        let clvalue = match response.stored_value {
            casper_client::types::StoredValue::CLValue(v) => v,
            _ => panic!("Expected CLValue."),
        };

        clvalue
    }

    pub async fn get_stored_clvalue_from_dict(
        &self,
        dictionary_seed_uref: &URef,
        dictionary_item_key: &str,
    ) -> CLValue {
        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await;

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
        .await
        .unwrap();
        let clvalue = match response.result.stored_value {
            casper_client::types::StoredValue::CLValue(v) => v,
            _ => panic!("Expected CLValue."),
        };

        clvalue
    }
}
