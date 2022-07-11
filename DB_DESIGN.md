# DB Enhancement Proposals

We have found some issues in Indexer DB.
In order to provide all the desired features to the API, we have some required and some aspirational changes.

## New requirements for any assets__* table:
1. enumeration column that goes through all the events. Required for all endpoints with pagination.  
   Let's say we can have max 10_000 of all the events per chunk  
   max 1000 of different standards  
   max 10_000 chunks  
We take timestamp in milliseconds to have more available slots for storing the info.  
We also multiply timestamp to a big number, so that we have 10^35 in general.  
It will allow us to change the format further without breaking the compatibility.  
   The formula:
```
timestamp_millis * 100_000_000_000 * 100_000_000_000 + chunk_index * 10_000_000 + assets_type * 10_000 + index_of_event
```
It gives us the number like this:
```
16565042430000000000000000030070027 -> 10^35
```
Major thing: it fits into `i128` (10^38).

## New requirements for assets__fungible_token_events table:
1. Add absolute value column. Required for balance request.

## New requirements for account_changes and balances table:
1. Change the table so that we have the same enumeration column. Required for native_history (pagination)  
   Let's say we can have max 1_000_000 of balance changing actions per chunk  
   max 10_000 chunks  
   The formula:
```
timestamp_millis * 10_000_000_000 * 1_000_000_000_000 + chunk_index * 1_000_000 + index_in_chunk
```
It gives us the number like this:
```
16565042430000000000000000003000027 -> 10^35
```
Major thing: it fits into `i128` (10^38).

## Open question 1. Do we want separate FT, MT tables or one Coins table?

### We want to have the one table because:
- We can easily serve the data about all coins with one query, sort, filter it. Both balances and history
- It's more convenient in general to have similar structure of what we store and what we serve

### We want to have separate tables because:
- If the other Coin standard appears, it would be easier just to create the new separate table with the needed structure
- For supporting MT, we need to add `symbol` column (or smth like that) to the table (it's easy, but anyway)

### Idea: even merge it to balances table
- Sounds reasonable, but I think it's not a great idea because NEAR balances table tends to be super huge

## Open question 2. Do we want to store all the tables in the format of affected/involved account_id?
- It's good to have the same format everywhere
- We use affected/involved in account_changes, we use from/to in assets.  
  affected/involved looks more convenient in the code

## Ideas how to make the queries faster:
1. Add table with the NEAR balance daily by account_id. Will speed up native_balance
2. Add table with the FT balance daily by account_id. Will speed up coin_balances
3. Add table with the MT balance daily by account_id. Will speed up coin_balances
4. Add table with the NFT countings daily by account_id. Will speed up nft_collection_overview
5. check_account_exists already works faster in Aurora DB. This will speed up all the endpoints a bit
6. Add table with FT, MT, NFT contract metadatas by timestamp. I'd prefer to check this once in an hour. Do we want to add something like "enforce checking new metadata"? Will speed up all the places where we provide metadata (almost all endpoints)
7. Add table with MT, NFT tokens metadata.
