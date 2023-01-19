# NEAR Enhanced API

API for providing useful information about NEAR blockchain.  
Still under heavy development.

### Supported features

- Provide NEAR balances information, history
- Provide FT balances information, FT history (*)
- Provide NFT information and recent history for the contracts implementing Events NEP
- Provide corresponding Metadata for FT, NFT contracts, NFT items

(*) We support all the FT contracts implementing Events NEP and some popular legacy contracts such as `aurora`, `wrap.near` and few others.
If your contract is not supported, please update with our new [SDK](https://github.com/near/near-sdk-rs).  
If it's important for you to collect all the previous history as well, you need to make the contribution and implement your own legacy handler.
You can use [existing handlers](https://github.com/near/near-microindexers/tree/main/indexer-events/src/db_adapters/coin/legacy) as the example.
