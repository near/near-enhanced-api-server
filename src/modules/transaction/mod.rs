use paperclip::actix::web;

// mod data_provider;
mod resources;
mod schemas;

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
    app.service(
        web::resource("/transaction/{transaction_hash}")
            .route(web::get().to(resources::get_transaction_by_tx_hash)),
    )
    .service(
        web::resource("/transaction/receipt/{receipt_id}")
            .route(web::get().to(resources::get_transaction_by_receipt)),
    )
    .service(
        web::resource("/transactions/{account_id}")
            .route(web::get().to(resources::get_transactions_by_account)),
    )
    .service(
        web::resource("/transactions/{account_id}/{contract_id}")
            .route(web::get().to(resources::get_transactions_by_account_on_contract)),
    )
    .service(
        web::resource("/receipts/{transaction_hash}")
            .route(web::get().to(resources::get_receipts_by_tx_hash)),
    )
    .service(
        web::resource("/receipts/actions/{account_id}")
            .route(web::get().to(resources::get_action_receipts_by_account)),
    )
    .service(
        web::resource("/receipts/actions/{account_id}/{contract_id}")
            .route(web::get().to(resources::get_action_receipts_by_account_on_contract)),
    );
}
