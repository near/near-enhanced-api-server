

## New requirements for any assets__* table:
1. enumeration column that goes through all the events. Required for coin_history
   Let's say we can have max 100_000 of all the events per chunk
   max 1000 of different assets
   max 1000 chunks
   The formula:
```
timestamp * 100_000_000_000 + chunk_index * 100_000_000 + assets_type * 100_000 + index_of_event
```
It gives us the number like this:
```
165364902022029300700300700027 -> 10^30
```
Fortunately, it fits into `u128` (10^38).


## New requirements for account_changes table:
1. Change the table so that we have the same enum column. Required for native_history
   Let's say we can have max 100_000_000 of balance changing actions
   max 1000 chunks
   The formula:
```
timestamp * 100_000_000_000 + chunk_index * 100_000_000 + index_in_chunk
```
It gives us the number like this:
```
165364902022029300700300000027 -> 10^30
```
Fortunately, it fits into `u128` (10^38).


## Ideas how to make the queries faster:
1. Add table with the NEAR balance daily by account_id (adding line only if the data changed during the day). Will speed up native_balance
2. Add table with the FT balance daily by account_id (adding line only if the data changed during the day). Will speed up coin_balances
3. Add table with the MT balance daily by account_id (adding line only if the data changed during the day). Will speed up coin_balances
4. Add table with the NFT countings daily by account_id (adding line only if the data changed during the day). Will speed up nft_balance_overview
5. check_account_exists already works faster in Aurora. This will speed up all the endpoints
6. Add table with FT, MT, NFT contract metadatas by timestamp. I'd prefer to check this daily. Do we want to add something like "enforce checking new metadata"? Will speed up ft_metadata, nft_metadata and other where we provide metadata
7. Add table with NFT token metadata. Will speed up nft methods
