# NEAR Enhanced API

API for providing useful information about NEAR blockchain.  
Still under heavy development.

### Phase 1 goals: [development goes here now]
- Provide NEAR balances information, recent history
- Provide FT balances information, recent FT coin history for the contracts implementing Events NEP
- Provide NFT information and recent history for the contracts implementing Events NEP
- Provide corresponding Metadata for FT, NFT contracts, NFT items
- Collect usage statistics which could help us to prioritize next steps

Note, Phase 1 will **not** provide pagination through all the history.
Phase 1 also does **not** provide the information about contracts which are not upgraded to Events NEP.

If you are interested in a more detailed development status, use `git clone` and search for `TODO PHASE 1`.

### Future plans. Phase 2 goals:
- Provide pagination for all existing endpoints where applicable
- Support contracts which are not upgraded to Events NEP, such as `wrap.near`, `aurora`
- Add reconciliation logic
- [aspirational] Support MT contracts 
- [aspirational] Support of querying the balance/history info by symbol (e.g. `GET "/accounts/{account_id}/coins/USN"`) 

### Future plans. Phase 3+ goals:
- Make wrappers around existing RPC endpoints for the general blockchain info (blocks, chunks, transactions, etc.)

## How to run it yourself

You need to create `.env` file with 3 variables: `DATABASE_URL`, `DATABASE_URL_BALANCES`, `RPC_URL`.
`DATABASE_URL_BALANCES` is a temp solution with the new table, it's under development.

All the other stuff is super standard for Rust world.  
To modify and then review tests, use `cargo insta review`.
