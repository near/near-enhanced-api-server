use paperclip::actix::web;

mod data_provider;
mod resources;
mod schemas;

pub(crate) fn register_services(app: &mut web::ServiceConfig) {
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
            .route(web::get().to(resources::get_nft)),
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
