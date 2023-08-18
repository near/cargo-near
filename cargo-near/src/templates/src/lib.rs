use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId};
use std::collections::HashMap;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StatusMessage {
    records: HashMap<AccountId, String>,
}

#[near_bindgen]
impl StatusMessage {
    #[payable]
    pub fn set_status(&mut self, message: String) {
        let account_id = env::signer_account_id();
        log!("{} set_status with message {}", account_id, message);
        self.records.insert(account_id, message);
    }

    pub fn get_status(&self, account_id: AccountId) -> Option<String> {
        log!("get_status for account_id {}", account_id);
        self.records.get(&account_id).cloned()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("bob_near".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    fn set_get_message() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = StatusMessage::default();
        contract.set_status("hello".to_string());
        assert_eq!(get_logs(), vec!["bob_near set_status with message hello"]);
        let context = get_context(true);
        testing_env!(context);
        assert_eq!("hello".to_string(), contract.get_status("bob_near".parse().unwrap()).unwrap());
        assert_eq!(get_logs(), vec!["get_status for account_id bob_near"])
    }

    #[test]
    fn get_nonexistent_message() {
        let context = get_context(true);
        testing_env!(context);
        let contract = StatusMessage::default();
        assert_eq!(None, contract.get_status("francis.near".parse().unwrap()));
        assert_eq!(get_logs(), vec!["get_status for account_id francis.near"])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::testing_env;
    use workspaces::Contract;
    use workspaces::TestRuntime;

    #[test]
    fn test_set_get_status() {
        // Setup the test runtime
        let mut test_runtime = TestRuntime::new();

        // Deploy the StatusMessage contract to the test runtime
        let contract_account_id = "status_contract".to_string();
        let contract = test_runtime.deploy(contract_account_id.clone(), "res/status_contract.wasm".to_string());

        // Call set_status
        contract.call(contract_account_id.clone(), "set_status", &b"{\"message\": \"Hello World\"}".to_vec(), 0);
        let response = contract.view(contract_account_id.clone(), "get_status", &b"{\"account_id\": \"status_contract\"}".to_vec());

        // Verify the response
        assert_eq!(response.as_string().unwrap(), "Hello World");
    }
}
