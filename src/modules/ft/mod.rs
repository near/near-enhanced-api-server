use paperclip::actix::web;

mod data_provider;
mod resources;
mod schemas;

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
    app.service(
        web::resource("/accounts/{account_id}/balances/FT")
            .route(web::get().to(resources::get_ft_balances)),
    )
    .service(
        web::resource("/accounts/{account_id}/balances/FT/{contract_account_id}")
            .route(web::get().to(resources::get_ft_balance_by_contract)),
    )
    .service(
        web::resource("/accounts/{account_id}/balances/FT/{contract_account_id}/history")
            .route(web::get().to(resources::get_ft_history)),
    )
    .service(
        web::resource("/nep141/metadata/{contract_account_id}")
            .route(web::get().to(resources::get_ft_metadata)),
    );
}
