use paperclip::actix::web;

// mod data_provider;
mod resources;
mod schemas;

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
    app.service(web::resource("/transaction").route(web::get().to(resources::get_transaction)))
        .service(web::resource("/transactions").route(web::get().to(resources::get_transactions)))
        .service(web::resource("/transactions/receipts").route(web::get().to(resources::get_receipts)))
        .service(
            web::resource("/transactions/receipts/actions").route(web::get().to(resources::get_action_receipts)),
        );
}
