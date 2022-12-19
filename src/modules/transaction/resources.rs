use crate::types;
use std::str::FromStr;

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
            transaction_hash: types::CryptoHash::from_str(
                "E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ",
            )
            .unwrap(),
            signer_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
            signer_public_key: types::PublicKey::from_str(
                "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK232",
            )
            .unwrap()
            .to_string(),
            receiver_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
            block_hash: types::CryptoHash::from_str("56qTxhPZosvJHazph2NbaQdUMJHA1P9poREV3Bw1JKEV")
                .unwrap(),
            activities: Vec::new(),
            timestamp: types::numeric::U64(1670017393533),
            total_gas_cost: types::numeric::U64(0),
            amount: types::numeric::U128(0),
            status: schemas::TxStatus::Pending,
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
    _request: actix_web_validator::Path<schemas::TransactionByReceiptId>,
) -> crate::Result<Json<schemas::TransactionResponse>> {
    Ok(Json(schemas::TransactionResponse {
        transaction: schemas::Transaction {
            transaction_hash: types::CryptoHash::from_str(
                "E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ",
            )
            .unwrap(),
            signer_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
            signer_public_key: types::PublicKey::from_str(
                "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK232",
            )
            .unwrap()
            .to_string(),
            receiver_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
            block_hash: types::CryptoHash::from_str("56qTxhPZosvJHazph2NbaQdUMJHA1P9poREV3Bw1JKEV")
                .unwrap(),
            activities: Vec::new(),
            timestamp: types::numeric::U64(1670017393533),
            total_gas_cost: types::numeric::U64(0),
            amount: types::numeric::U128(0),
            status: schemas::TxStatus::Pending,
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
        transaction_hash: types::CryptoHash::from_str(
            "E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ",
        )
        .unwrap(),
        signer_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        signer_public_key: types::PublicKey::from_str(
            "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK232",
        )
        .unwrap()
        .to_string(),
        receiver_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        block_hash: types::CryptoHash::from_str("56qTxhPZosvJHazph2NbaQdUMJHA1P9poREV3Bw1JKEV")
            .unwrap(),
        activities: Vec::new(),
        timestamp: types::numeric::U64(1670017393533),
        total_gas_cost: types::numeric::U64(0),
        amount: types::numeric::U128(0),
        status: schemas::TxStatus::Pending,
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
        transaction_hash: types::CryptoHash::from_str(
            "E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ",
        )
        .unwrap(),
        signer_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        signer_public_key: types::PublicKey::from_str(
            "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK232",
        )
        .unwrap()
        .to_string(),
        receiver_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        block_hash: types::CryptoHash::from_str("56qTxhPZosvJHazph2NbaQdUMJHA1P9poREV3Bw1JKEV")
            .unwrap(),
        activities: Vec::new(),
        timestamp: types::numeric::U64(1670017393533),
        total_gas_cost: types::numeric::U64(0),
        amount: types::numeric::U128(0),
        status: schemas::TxStatus::Pending,
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
    let activities = vec![schemas::Activity {
        receipt_id: types::CryptoHash::from_str("E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ")
            .unwrap(),
        signer_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        signer_public_key: types::PublicKey::from_str(
            "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK232",
        )
        .unwrap()
        .to_string(),
        operations: vec![],
        predecessor_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        receiver_account_id: types::AccountId::from_str("spot.spin-fi.near").unwrap(),
        status: schemas::ActivityStatus::Pending,
        logs: vec![],
    }];
    Ok(Json(schemas::ReceiptsResponse { activities }))
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
    let action_activities = vec![schemas::Activity {
        receipt_id: types::CryptoHash::from_str("E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ")
            .unwrap(),
        signer_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        signer_public_key: types::PublicKey::from_str(
            "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK232",
        )
        .unwrap()
        .to_string(),
        operations: vec![],
        predecessor_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        receiver_account_id: types::AccountId::from_str("spot.spin-fi.near").unwrap(),
        status: schemas::ActivityStatus::Pending,
        logs: vec![],
    }];
    Ok(Json(schemas::ActionReceiptsResponse {
        activities: action_activities,
    }))
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
    let action_activities = vec![schemas::Activity {
        receipt_id: types::CryptoHash::from_str("E2gtnNchwDrLUL7prNSdfcUzwwR4egJV4qpncwHz1hwJ")
            .unwrap(),
        signer_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        signer_public_key: types::PublicKey::from_str(
            "232232TxhPZosvJHsdfsfsdf2UMJHA1P9poRBw1JK232",
        )
        .unwrap()
        .to_string(),
        operations: vec![],
        predecessor_account_id: types::AccountId::from_str("roshaan.near").unwrap(),
        receiver_account_id: types::AccountId::from_str("spot.spin-fi.near").unwrap(),
        status: schemas::ActivityStatus::Pending,
        logs: vec![],
    }];
    Ok(Json(schemas::ActionReceiptsResponse {
        activities: action_activities,
    }))
}
