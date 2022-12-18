use crate::types;

use super::schemas;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

#[api_v2_operation(tags(Transaction))]
/// Get transaction by tx hash
///
/// This endpoint returns the details of a transaction given a `transaction_hash`
pub async fn get_transaction_by_tx_hash(
    _pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    _rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    _request: actix_web_validator::Path<schemas::TransactionByTxHash>,
) -> crate::Result<Json<schemas::TransactionResponse>> {
    Ok(Json(schemas::TransactionResponse {
        transaction: schemas::Transaction {
            transaction_hash: "E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ".to_string(),
            signer_account_id: "roshaan.near".to_string(),
            signer_public_key: "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK23".to_string(),
            receiver_account_id: "roshaan.near".to_string(),
            block_hash: "56qTxhPZosvJHazph2NbaQdUMJHA1P9poREV3Bw1JKEV".to_string(),
            actions: Vec::new(),
            timestamp: 1670017393533,
            total_gas_cost: 0_u128,
            amount: 0_u128,
            status: "success".to_string(),
        },
    }))
}

#[api_v2_operation(tags(Transaction))]
/// Get transaction by receipt id
///
/// This endpoint returns the details of a transaction given a `receipt_id`
pub async fn get_transaction_by_receipt(
    _pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    _rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    _request: actix_web_validator::Path<schemas::TransactionByTxHash>,
) -> crate::Result<Json<schemas::TransactionResponse>> {
    Ok(Json(schemas::TransactionResponse {
        transaction: schemas::Transaction {
            transaction_hash: "E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ".to_string(),
            signer_account_id: "roshaan.near".to_string(),
            signer_public_key: "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK23".to_string(),
            receiver_account_id: "roshaan.near".to_string(),
            block_hash: "56qTxhPZosvJHazph2NbaQdUMJHA1P9poREV3Bw1JKEV".to_string(),
            actions: Vec::new(),
            timestamp: 1670017393533,
            total_gas_cost: 0_u128,
            amount: 0_u128,
            status: "success".to_string(),
        },
    }))
}

#[api_v2_operation(tags(Transaction))]
/// Get user's transaction history
///
/// This endpoint returns the history of transactions made by an `account_id`
/// Additonally, you can specify `contract_id` to retrieve transactions that a given `account_id` performed on a Near
/// `contract_id`
///
/// This endpoint supports pagination
pub async fn get_transactions_by_account(
    _pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    _rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    _request: actix_web_validator::Path<schemas::TransactionsByAccountId>,
    _pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::TransactionsResponse>> {
    let mut transactions: Vec<schemas::Transaction> = Vec::new();
    let transaction = schemas::Transaction {
        transaction_hash: "E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ".to_string(),
        signer_account_id: "roshaan.near".to_string(),
        signer_public_key: "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK23".to_string(),
        receiver_account_id: "roshaan.near".to_string(),
        block_hash: "56qTxhPZosvJHazph2NbaQdUMJHA1P9poREV3Bw1JKEV".to_string(),
        actions: Vec::new(),
        timestamp: 1670017393533,
        total_gas_cost: 0_u128,
        amount: 0_u128,
        status: "success".to_string(),
    };
    transactions.push(transaction.clone());
    transactions.push(transaction);
    Ok(Json(schemas::TransactionsResponse { transactions }))
}

#[api_v2_operation(tags(Transaction))]
/// Get user's transaction history on contract
///
/// This endpoint returns the history of transactions made by an `account_id` on a specific `contract_id`
///
/// This endpoint supports pagination
pub async fn get_transactions_by_account_on_contract(
    _pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    _rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    _request: actix_web_validator::Path<schemas::TransactionsByAccountIdOnContract>,
    _pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::TransactionsResponse>> {
    let mut transactions: Vec<schemas::Transaction> = Vec::new();
    let transaction = schemas::Transaction {
        transaction_hash: "E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ".to_string(),
        signer_account_id: "roshaan.near".to_string(),
        signer_public_key: "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK23".to_string(),
        receiver_account_id: "roshaan.near".to_string(),
        block_hash: "56qTxhPZosvJHazph2NbaQdUMJHA1P9poREV3Bw1JKEV".to_string(),
        actions: Vec::new(),
        timestamp: 1670017393533,
        total_gas_cost: 0_u128,
        amount: 0_u128,
        status: "success".to_string(),
    };
    transactions.push(transaction.clone());
    transactions.push(transaction);
    Ok(Json(schemas::TransactionsResponse { transactions }))
}

#[api_v2_operation(tags(Transaction))]
/// Get transaction receipts
///
/// This endpoint will retrieve an ordered list of receipts generated by a `transaction_hash`
pub async fn get_receipts_by_tx_hash(
    _pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    _rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    _request: actix_web_validator::Path<schemas::ReceiptsByTxHash>,
    _pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::ReceiptsResponse>> {
    let mut receipts: Vec<schemas::Receipt> = Vec::new();
    let action = schemas::ActionReceipt {
        signer_account_id: "roshaan.near".to_string(),
        signer_public_key: "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK23".to_string(),
        gas_price: types::numeric::U128(0_u128),
        actions: vec![schemas::ActionType::CreateAccount(
            schemas::CreateAccountAction {},
        )],
    };
    let receipt = schemas::Receipt {
        receipt_id: "APFoQw6Hc2pJTZyYJw3tYLSdHjb8poacH7eYL5gK2W8n".to_string(),
        originated_from_transaction_hash: Some(
            "GcajpeVRUbhLdHN8UpDTUZV8YYBdcRtLsTwzwWZq6MDi".to_string(),
        ),
        predecessor_account_id: "roshaan.near".to_string(),
        receiver_account_id: "spot.spin-fi.near".to_string(),
        actions: vec![action],
        receipt_kind: "action".to_string(),
        status: "success".to_string(),
        block_timestamp: Some(66862877),
        gas_burnt: Some(types::numeric::U128(223_u128)),
        tokens_burnt: Some(types::numeric::U128(0.00083 as u128)),
    };
    receipts.push(receipt);
    Ok(Json(schemas::ReceiptsResponse { receipts }))
}

#[api_v2_operation(tags(Transaction))]
/// Get user's actions
///
/// This endpoint will retrieve an ordered list of actions performed by an `account_id`
///
pub async fn get_action_receipts_by_account(
    _pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    _rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    _request: actix_web_validator::Path<schemas::ActionReceiptsByAccountId>,
    _pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::ActionReceiptsResponse>> {
    let mut action_receipts: Vec<schemas::Receipt> = Vec::new();
    let action = schemas::ActionReceipt {
        signer_account_id: "roshaan.near".to_string(),
        signer_public_key: "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK23".to_string(),
        gas_price: types::numeric::U128(0_u128),
        actions: vec![schemas::ActionType::CreateAccount(
            schemas::CreateAccountAction {},
        )],
    };
    let receipt = schemas::Receipt {
        receipt_id: "APFoQw6Hc2pJTZyYJw3tYLSdHjb8poacH7eYL5gK2W8n".to_string(),
        originated_from_transaction_hash: Some(
            "GcajpeVRUbhLdHN8UpDTUZV8YYBdcRtLsTwzwWZq6MDi".to_string(),
        ),
        predecessor_account_id: "roshaan.near".to_string(),
        receiver_account_id: "spot.spin-fi.near".to_string(),
        actions: vec![action],
        receipt_kind: "action".to_string(),
        status: "success".to_string(),
        block_timestamp: Some(66862877),
        gas_burnt: Some(types::numeric::U128(223_u128)),
        tokens_burnt: Some(types::numeric::U128(0.00083 as u128)),
    };
    action_receipts.push(receipt);
    Ok(Json(schemas::ActionReceiptsResponse { action_receipts }))
}

#[api_v2_operation(tags(Transaction))]
/// Get user's actions on contract
///
/// This endpoint will retrieve an ordered list of actions performed by an `account_id` on a specific `contract_id`
///

pub async fn get_action_receipts_by_account_on_contract(
    _pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    _rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    _request: actix_web_validator::Path<schemas::ActionReceiptsByAccountIdOnContract>,
    _pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::ActionReceiptsResponse>> {
    let mut action_receipts: Vec<schemas::Receipt> = Vec::new();
    let action = schemas::ActionReceipt {
        signer_account_id: "roshaan.near".to_string(),
        signer_public_key: "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK23".to_string(),
        gas_price: types::numeric::U128(0_u128),
        actions: vec![schemas::ActionType::CreateAccount(
            schemas::CreateAccountAction {},
        )],
    };
    let receipt = schemas::Receipt {
        receipt_id: "APFoQw6Hc2pJTZyYJw3tYLSdHjb8poacH7eYL5gK2W8n".to_string(),
        originated_from_transaction_hash: Some(
            "GcajpeVRUbhLdHN8UpDTUZV8YYBdcRtLsTwzwWZq6MDi".to_string(),
        ),
        predecessor_account_id: "roshaan.near".to_string(),
        receiver_account_id: "spot.spin-fi.near".to_string(),
        actions: vec![action],
        receipt_kind: "action".to_string(),
        status: "success".to_string(),
        block_timestamp: Some(66862877),
        gas_burnt: Some(types::numeric::U128(223_u128)),
        tokens_burnt: Some(types::numeric::U128(0.00083 as u128)),
    };
    action_receipts.push(receipt);
    Ok(Json(schemas::ActionReceiptsResponse { action_receipts }))
}
