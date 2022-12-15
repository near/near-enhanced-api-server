use paperclip::actix::web;

mod data_provider;
mod resources;
mod schemas;

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
    // I'll drop these comments, leaving it only because it's easier for the review
    // app.service(
    //     web::resource("/accounts/{account_id}/NEAR")
    //         .route(web::get().to(resources::get_near_balance)),
    // )
    app.service(
        web::resource("/accounts/{account_id}/FT").route(web::get().to(resources::get_ft_balances)),
    )
    .service(
        web::resource("/accounts/{account_id}/FT/{contract_account_id}")
            .route(web::get().to(resources::get_ft_balance_by_contract)),
    )
    // .service(
    //     web::resource("/accounts/{account_id}/NEAR/history")
    //         .route(web::get().to(resources::get_near_history)),
    // )
    .service(
        web::resource("/accounts/{account_id}/FT/{contract_account_id}/history")
            .route(web::get().to(resources::get_ft_history)),
    )
    .service(
        web::resource("/nep141/metadata/{contract_account_id}")
            .route(web::get().to(resources::get_ft_metadata)),
    );
}
