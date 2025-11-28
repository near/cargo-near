// Find all our documentation at https://docs.near.org
use near_sdk::json_types::U64;
use near_sdk::{env, near, require, AccountId, NearToken, PanicOnDefault, Promise};

// Define the contract structure
#[near(contract_state)]
#[derive(PanicOnDefault)] // The contract is required to be initialized with either init or init(ignore_state) methods
pub struct Contract {
    highest_bid: Bid,
    auction_end_time: U64,
    auctioneer: AccountId,
    claimed: bool,
}

// Define the Bid structure
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Bid {
    pub bidder: AccountId,
    pub bid: NearToken,
}

// Implement the contract structure
#[near]
impl Contract {
    #[init]
    #[private] // Private method - only callable by the contract's account
    pub fn init(end_time: U64, auctioneer: AccountId) -> Self {
        Self {
            highest_bid: Bid {
                bidder: env::current_account_id(),
                bid: NearToken::from_yoctonear(1),
            },
            auction_end_time: end_time,
            claimed: false,
            auctioneer,
        }
    }

    // Public method - bids on the auction
    #[payable]
    pub fn bid(&mut self) -> Promise {
        // Assert the auction is still ongoing
        require!(
            env::block_timestamp() < self.auction_end_time.into(),
            "Auction has ended"
        );

        // Current bid
        let bid = env::attached_deposit();
        let bidder = env::predecessor_account_id();

        // Last bid
        let Bid {
            bidder: last_bidder,
            bid: last_bid,
        } = self.highest_bid.clone();

        // Check if the deposit is higher than the current bid
        require!(bid > last_bid, "You must place a higher bid");

        // Update the highest bid
        self.highest_bid = Bid { bidder, bid };

        // Transfer tokens back to the last bidder
        Promise::new(last_bidder).transfer(last_bid)
    }

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

    /*
     Public methods - returns the highest bid, auction end time, auctioneer, and claimed status
    */
    pub fn get_highest_bid(&self) -> Bid {
        self.highest_bid.clone()
    }

    pub fn get_auction_end_time(&self) -> U64 {
        self.auction_end_time
    }

    pub fn get_auctioneer(&self) -> AccountId {
        self.auctioneer.clone()
    }

    pub fn get_claimed(&self) -> bool {
        self.claimed
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
    fn init_contract() {
        let end_time: U64 = U64::from(1000);
        let alice: AccountId = "alice.near".parse().unwrap();
        let contract = Contract::init(end_time.clone(), alice.clone());

        let default_bid = contract.get_highest_bid();
        assert_eq!(default_bid.bidder, env::current_account_id());
        assert_eq!(default_bid.bid, NearToken::from_yoctonear(1));

        let auction_end_time = contract.get_auction_end_time();
        assert_eq!(auction_end_time, end_time);

        let auctioneer = contract.get_auctioneer();
        assert_eq!(auctioneer, alice);

        let claimed = contract.get_claimed();
        assert_eq!(claimed, false);
    }

    #[test]
    fn bid_successfully() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time.clone(), auctioneer.clone());

        // Set block_timestamp before auction end time
        let mut context = get_context(alice.clone());
        context.block_timestamp(500);
        context.attached_deposit(NearToken::from_near(2));
        testing_env!(context.build());

        // Bid should succeed
        contract.bid();

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
        let mut contract = Contract::init(end_time.clone(), auctioneer.clone());

        // Set block_timestamp after auction end time
        let mut context = get_context(alice.clone());
        context.block_timestamp(2000);
        context.attached_deposit(NearToken::from_near(2));
        testing_env!(context.build());

        // Bid should panic
        contract.bid();
    }

    #[test]
    #[should_panic(expected = "You must place a higher bid")]
    fn bid_lower_than_current() {
        let auctioneer: AccountId = "auctioneer.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let end_time: U64 = U64::from(1000);
        let mut contract = Contract::init(end_time.clone(), auctioneer.clone());

        // Set block_timestamp before auction end time
        let mut context = get_context(alice.clone());
        context.block_timestamp(500);
        // Default bid is 1 yoctoNEAR, so bidding with 0 or less should fail
        // But we'll bid with the same amount (1 yoctoNEAR) which should also fail
        context.attached_deposit(NearToken::from_yoctonear(1));
        testing_env!(context.build());

        // Bid should panic
        contract.bid();
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
