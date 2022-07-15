use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use paperclip::actix::{App, web};

mod schemas;
mod resources;
mod data_provider;

pub(crate) fn register_service<T, U, V, X>(app: &mut App<dyn ServiceFactory<ServiceRequest, Response=ServiceResponse<T>, Error=U, Config=(), Service=V, InitError=(), Future=X>>) {
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
                .route(web::get().to(resources::get_balances_by_contract)),
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
