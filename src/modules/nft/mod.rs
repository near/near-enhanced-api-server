use actix_web::dev::{ServiceFactory, ServiceRequest};
use paperclip::actix::{App, web};

mod schemas;
mod data_provider;
mod resources;

pub(crate) fn register_service<T>(app: &mut App<T>) where T: ServiceFactory<ServiceRequest> {
    app.service(
            web::resource("/accounts/{account_id}/NFT")
                .route(web::get().to(resources::get_nft_collection_overview)),
        )
        .service(
            web::resource("/accounts/{account_id}/NFT/{contract_account_id}")
                .route(web::get().to(resources::get_nft_collection_by_contract)),
        )
        .service(
            web::resource("/NFT/{contract_account_id}/{token_id}")
                .route(web::get().to(resources::get_nft_item_details)),
        )
        .service(
            web::resource("/NFT/{contract_account_id}/{token_id}/history")
                .route(web::get().to(resources::get_nft_history)),
        )
        .service(
            web::resource("/nep171/metadata/{contract_account_id}")
                .route(web::get().to(resources::get_nft_contract_metadata)),
        );
}
