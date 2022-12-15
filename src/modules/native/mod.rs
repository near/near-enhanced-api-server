use paperclip::actix::web;

mod data_provider;
mod resources;
mod schemas;

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
    app.service(
        web::resource("/accounts/{account_id}/balances/NEAR")
            .route(web::get().to(resources::get_near_balance)),
    )
    .service(
        web::resource("/accounts/{account_id}/balances/NEAR/history")
            .route(web::get().to(resources::get_near_history)),
    );
}
