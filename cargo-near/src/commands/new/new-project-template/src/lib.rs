// Find NEAR documentation at https://docs.near.org
use near_sdk::json_types::U64;
use near_sdk::{AccountId, NearToken, PanicOnDefault, Promise, Timestamp, env, near, require};

// Define the contract structure
#[near(contract_state)]
#[derive(PanicOnDefault)] // The contract is required to be initialized with `#[init]` functions
pub struct Contract {
    highest_bid: Bid,
    auction_end_time: Timestamp,
    auctioneer: AccountId,
    is_claimed: bool,
}

// The Bid structure is used as function return value (JSON-serialized) and as part of the Contract
// state (Borsh-serialized)
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Bid {
    pub bidder: AccountId,
    pub bid: NearToken,
}

// Implement the contract functions
#[near]
impl Contract {
    /// Initializer function that one must call after the contract code is deployed to an account
    /// for the first time, all other functions will fail to execute until the contract state is
    /// initialized (thanks to PanicOnDefault derive above).
    /// It is common to batch contract initialization in the same transaction as contract deployment.
    /// Sometimes #[private] attribute can also be useful to guard the function to be callable only
    /// by the account where the contract is deployed to.
    #[init]
    pub fn init(end_time: U64, auctioneer: AccountId) -> Self {
        Self {
            highest_bid: Bid {
                bidder: env::current_account_id(),
                bid: NearToken::from_yoctonear(1),
            },
            auction_end_time: end_time.into(),
            is_claimed: false,
            auctioneer,
        }
    }

    /// Bid function can be called by any account on blockchain to make a higher bid on the auction
    #[payable]
    pub fn bid(&mut self) -> Promise {
        // Assert the auction is still ongoing
        require!(
            env::block_timestamp() < self.auction_end_time,
            "Auction has ended"
        );

        // Current bid
        let bid = env::attached_deposit();
        let bidder = env::predecessor_account_id();

        // Last bid recorded by the contract
        let Bid {
            bidder: last_bidder,
            bid: last_bid,
        } = self.highest_bid.clone();

        // Check if the deposit is higher than the current bid
        require!(bid > last_bid, "You must place a higher bid");

        // Update the highest bid
        self.highest_bid = Bid { bidder, bid };

        // Transfer tokens back to the last bidder.
        //
        // NOTE: The result of this Promise is not handled. If this transfer fails (for example,
        // because `last_bidder` account was removed), the previous bidder may not be refunded even
        // though `self.highest_bid` has already been updated. For production use, consider
        // implementing a withdrawal pattern or adding a callback to handle transfer failures.
        Promise::new(last_bidder).transfer(last_bid)
    }

    /// Claim function can be called by any account on blockchain to claim the auction and transfer
    /// the tokens to the auctioneer
    pub fn claim(&mut self) -> Promise {
        // Assert the auction has ended
        require!(
            env::block_timestamp() > self.auction_end_time,
            "Auction has not ended yet"
        );

        // Assert the auction has not been claimed yet
        require!(!self.is_claimed, "Auction has already been claimed");
        self.is_claimed = true;

        // Transfer tokens to the auctioneer.
        //
        // NOTE: The result of this Promise is not handled. If this transfer fails (for example,
        // because `last_bidder` account was removed), the previous bidder may not be refunded even
        // though `self.highest_bid` has already been updated. For production use, consider
        // implementing a withdrawal pattern or adding a callback to handle transfer failures.
        Promise::new(self.auctioneer.clone()).transfer(self.highest_bid.bid)
    }

    /*
     * The functions below are read-only functions that can be called without a transaction (through
     * JSON RPC query call). They read the data from the contract local storage and return the
     * highest bid, auction end time, auctioneer, and claimed status
     */

    pub fn get_highest_bid(&self) -> Bid {
        self.highest_bid.clone()
    }

    pub fn get_auction_end_time(&self) -> U64 {
        self.auction_end_time.into()
    }

    pub fn get_auctioneer(&self) -> AccountId {
        self.auctioneer.clone()
    }

    pub fn is_already_claimed(&self) -> bool {
        self.is_claimed
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
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::{AccountId, testing_env};

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn init_contract() {
        let end_time: U64 = U64::from(1000);
        let alice: AccountId = "alice.near".parse().unwrap();
        let contract = Contract::init(end_time, alice.clone());

        let default_bid = contract.get_highest_bid();
        assert_eq!(default_bid.bidder, env::current_account_id());
        assert_eq!(default_bid.bid, NearToken::from_yoctonear(1));

        let auction_end_time = contract.get_auction_end_time();
        assert_eq!(auction_end_time, end_time);

        let auctioneer = contract.get_auctioneer();
        assert_eq!(auctioneer, alice);

        assert!(!contract.is_already_claimed());
    }

    #[test]
    fn bid_successfully() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time, auctioneer.clone());

        // Set block_timestamp before auction end time
        let mut context = get_context(alice.clone());
        context.block_timestamp(500);
        context.attached_deposit(NearToken::from_near(2));
        testing_env!(context.build());

        // Bid should succeed
        let _ = contract.bid();

        // Verify highest bid is updated
        let highest_bid = contract.get_highest_bid();
        assert_eq!(highest_bid.bidder, alice);
        assert_eq!(highest_bid.bid, NearToken::from_near(2));
    }

    #[test]
    #[should_panic(expected = "Auction has ended")]
    fn bid_after_auction_ended() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time, auctioneer.clone());

        // Set block_timestamp after auction end time
        let mut context = get_context(alice.clone());
        context.block_timestamp(2000);
        context.attached_deposit(NearToken::from_near(2));
        testing_env!(context.build());

        // Bid should panic
        let _ = contract.bid();
    }

    #[test]
    #[should_panic(expected = "You must place a higher bid")]
    fn bid_lower_than_current() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time, auctioneer.clone());

        // Set block_timestamp before auction end time
        let mut context = get_context(alice.clone());
        context.block_timestamp(500);
        // Default bid is 1 yoctoNEAR, so bidding with 0 or less should fail
        // But we'll bid with the same amount (1 yoctoNEAR) which should also fail
        context.attached_deposit(NearToken::from_yoctonear(1));
        testing_env!(context.build());

        // Bid should panic
        let _ = contract.bid();
    }

    #[test]
    fn claim_after_auction_ended() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time, auctioneer.clone());

        // Set block_timestamp after auction end time
        let mut context = get_context(auctioneer.clone());
        context.block_timestamp(2000);
        testing_env!(context.build());

        // Claim should succeed
        let _ = contract.claim();

        // Verify auction is marked as claimed
        assert!(contract.is_already_claimed());
    }

    #[test]
    #[should_panic(expected = "Auction has not ended yet")]
    fn claim_before_auction_ended() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time, auctioneer.clone());

        // Set block_timestamp before auction end time
        let mut context = get_context(auctioneer.clone());
        context.block_timestamp(500);
        testing_env!(context.build());

        // Claim should panic
        let _ = contract.claim();
    }

    #[test]
    #[should_panic(expected = "Auction has already been claimed")]
    fn claim_twice() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time, auctioneer.clone());

        // Set block_timestamp after auction end time
        let mut context = get_context(auctioneer.clone());
        context.block_timestamp(2000);
        testing_env!(context.build());

        // First claim should succeed
        let _ = contract.claim();

        // Second claim should panic
        let _ = contract.claim();
    }
}
