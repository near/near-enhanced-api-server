use paperclip::actix::web;

mod data_provider;
mod resources;
mod schemas;

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
    app.service(
        web::resource("/accounts/{account_id}/coins/NEAR")
            .route(web::get().to(resources::get_near_balance)),
    )
    .service(
        web::resource("/accounts/{account_id}/coins")
            .route(web::get().to(resources::get_coin_balances)),
    )
    .service(
        web::resource("/accounts/{account_id}/coins/{contract_account_id}")
            .route(web::get().to(resources::get_coin_balances_by_contract)),
    )
    .service(
        web::resource("/accounts/{account_id}/coins/NEAR/history")
            .route(web::get().to(resources::get_near_history)),
    )
    .service(
        web::resource("/accounts/{account_id}/coins/{contract_account_id}/history")
            .route(web::get().to(resources::get_coin_history)),
    )
    .service(
        web::resource("/nep141/metadata/{contract_account_id}")
            .route(web::get().to(resources::get_ft_contract_metadata)),
    );
}
