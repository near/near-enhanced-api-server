/// This security schema is just used for OpenAPI spec declaration as validation of the api key
/// happens before the requests hit this service.
#[derive(paperclip::actix::Apiv2Security)]
#[openapi(
    apiKey,
    in = "header",
    name = "x-api-key",
    description = "Use Pagoda DevConsole API key here"
)]
pub struct PagodaApiKey;

impl actix_web::FromRequest for PagodaApiKey {
    type Error = actix_web::Error;
    type Future = futures::future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        _: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        // This is no-op by design as the access key gets validated on the API gateway level before
        // hitting Enhanced API server, thus we don't need to validate the key here at all.
        futures::future::ready(Ok(Self {}))
    }
}
