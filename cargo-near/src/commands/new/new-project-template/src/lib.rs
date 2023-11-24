use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct StatusMessage {
    records: LookupMap<AccountId, String>,
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    StatusMessageRecords,
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self {
            records: LookupMap::new(StorageKey::StatusMessageRecords),
        }
    }
}

#[near_bindgen]
impl StatusMessage {
    pub fn set_status(&mut self, message: String) {
        let account_id = env::predecessor_account_id();
        self.records.insert(&account_id, &message);
    }

    pub fn get_status(&self, account_id: AccountId) -> Option<String> {
        self.records.get(&account_id)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    // Allows for modifying the environment of the mocked blockchain
    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn set_get_message() {
        let mut context = get_context(accounts(1));
        // Initialize the mocked blockchain
        testing_env!(context.build());

        // Set the testing environment for the subsequent calls
        testing_env!(context.predecessor_account_id(accounts(1)).build());

        let mut contract = StatusMessage::default();
        contract.set_status("hello".to_string());
        assert_eq!(
            "hello".to_string(),
            contract.get_status(accounts(1)).unwrap()
        );
    }

    #[test]
    fn get_nonexistent_message() {
        let contract = StatusMessage::default();
        assert_eq!(None, contract.get_status("francis.near".parse().unwrap()));
    }
}
