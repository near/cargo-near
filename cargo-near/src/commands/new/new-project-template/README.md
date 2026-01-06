# Basic Auction Contract 

This directory contains a Rust contract that is used as part of the [Basic Auction Tutorial](https://docs.near.org/tutorials/auction/basic-auction).

The contract is a simple auction where you can place bids, view the highest bid, and claim the tokens at the end of the auction.

This repo showcases the basic anatomy of a smart contract application on NEAR Protocol that showcases how to store data in a contract, how to update the state, and then how to read it.
There are also unit tests, integration tests, and CI/CD pipelines preconfigured for your reference.

---

## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

If you deploy for debugging purposes:

```bash
cargo near deploy build-non-reproducible-wasm
```

If you deploy production ready smart contract:

```bash
cargo near deploy build-reproducible-wasm
```

## Initialize the contract

```bash
# on Linux / Windows WSL
TWO_MINUTES_FROM_NOW=$(date -d '+2 minutes' +%s000000000)
# on MacOS
TWO_MINUTES_FROM_NOW=$(date -v+2M +%s000000000)

near contract call-function as-transaction '<CONTRACT_ACCOUNT_ID>' init json-args '{"end_time": "'$TWO_MINUTES_FROM_NOW'", "auctioneer": "<AUCTIONEER_ACCOUNT_ID>"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR'
```

## Useful Links

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)
- [NEAR Documentation](https://docs.near.org)
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- [NEAR Discord](https://near.chat)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)
