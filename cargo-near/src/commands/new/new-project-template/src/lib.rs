// Find all our documentation at https://docs.near.org
use bid::Bid;
use near_sdk::json_types::U64;
use near_sdk::{env, near, AccountId, NearToken, PanicOnDefault};

mod bid;
mod claim;

// Define the contract structure
#[near(contract_state)]
#[derive(PanicOnDefault)] // The contract is required to be initialized with either init or init(ignore_state) methods
pub struct Contract {
    highest_bid: Bid,
    auction_end_time: U64,
    auctioneer: AccountId,
    claimed: bool,
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
}
