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
        futures::future::ready(Ok(Self {}))
    }
}
