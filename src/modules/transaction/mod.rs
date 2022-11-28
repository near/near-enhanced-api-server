use paperclip::actix::web;

// mod data_provider;
mod resources;
mod schemas;

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
    app.service(
        web::resource("/transactions/{tx_hash}").route(web::get().to(resources::get_tx_by_tx_hash)),
    )
    .service(
        web::resource("/transactions/{receipt_id}")
            .route(web::get().to(resources::get_tx_by_receipt_id)),
    )
    .service(
        web::resource("/transactions/{account_id}")
            .route(web::get().to(resources::get_txs_by_account_id)),
    )
    .service(
        web::resource("/transactions/{contract_id}/{account_id}")
            .route(web::get().to(resources::get_txs_by_account_id_on_contract_id)),
    )
    .service(
        web::resource("/transactions/receipts/{tx_hash}")
            .route(web::get().to(resources::get_tx_receipts_by_tx_hash)),
    )
    .service(
        web::resource("/transactions/{contract_id}/{account_id}")
            .route(web::get().to(resources::get_actions_by_account_id_on_contract_id)),
    )
    .service(
        web::resource("/transactions/{account_id}")
            .route(web::get().to(resources::get_actions_by_account_id)),
    );
}
