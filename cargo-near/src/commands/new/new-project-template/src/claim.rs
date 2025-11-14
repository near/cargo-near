use crate::{Contract, ContractExt};
use near_sdk::{env, near, require, Promise};

// Extend the contract implementation
#[near]
impl Contract {
    // Public method - claims the auction and transfers the tokens to the auctioneer
    pub fn claim(&mut self) -> Promise {
        // Assert the auction has ended
        require!(
            env::block_timestamp() > self.auction_end_time.into(),
            "Auction has not ended yet"
        );

        // Assert the auction has not been claimed yet
        require!(!self.claimed, "Auction has already been claimed");
        self.claimed = true;

        // Transfer tokens to the auctioneer
        Promise::new(self.auctioneer.clone()).transfer(self.highest_bid.bid)
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::json_types::U64;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn claim_after_auction_ended() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time.clone(), auctioneer.clone());

        // Set block_timestamp after auction end time
        let mut context = get_context(auctioneer.clone());
        context.block_timestamp(2000);
        testing_env!(context.build());

        // Claim should succeed
        contract.claim();

        // Verify auction is marked as claimed
        assert_eq!(contract.get_claimed(), true);
    }

    #[test]
    #[should_panic(expected = "Auction has not ended yet")]
    fn claim_before_auction_ended() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time.clone(), auctioneer.clone());

        // Set block_timestamp before auction end time
        let mut context = get_context(auctioneer.clone());
        context.block_timestamp(500);
        testing_env!(context.build());

        // Claim should panic
        contract.claim();
    }

    #[test]
    #[should_panic(expected = "Auction has already been claimed")]
    fn claim_twice() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time.clone(), auctioneer.clone());

        // Set block_timestamp after auction end time
        let mut context = get_context(auctioneer.clone());
        context.block_timestamp(2000);
        testing_env!(context.build());

        // First claim should succeed
        contract.claim();

        // Second claim should panic
        contract.claim();
    }
}
