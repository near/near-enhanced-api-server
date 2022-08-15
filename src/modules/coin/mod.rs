use actix_web::{HttpResponse, error};
use actix_web_validator::{Error, PathConfig};
use paperclip::actix::web;

mod data_provider;
mod resources;
mod schemas;

#[derive(serde::Serialize)]
pub struct ValidationErrorJsonPayload {
    pub message: String,
    pub fields: Vec<String>,
}

/// Custom error handler
impl From<&validator::ValidationErrors> for ValidationErrorJsonPayload {
    fn from(error: &validator::ValidationErrors) -> Self {
        ValidationErrorJsonPayload {
            message: "Validation error".to_owned(),
            fields: error.field_errors().iter().map(|(field, _)| field.to_string()).collect(),
        }
    }
}

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
    app
        .app_data(PathConfig::default().error_handler(|err, _| {
            let json_error = match &err {
                Error::Validate(error) => ValidationErrorJsonPayload::from(error),
                _ => ValidationErrorJsonPayload { message: err.to_string(), fields: Vec::new() },
            };
            error::InternalError::from_response(err, HttpResponse::Conflict().json(json_error)).into()
        }))
        .service(
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
