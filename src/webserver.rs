use actix_web::{get, web, App, HttpRequest, HttpServer, Responder};

use crate::{
    get_config,
    handlers::{
        estimate::{expiration_info, name, register_request, renew_request},
        get_plain_name, get_price, get_registration, img_gen,
    },
};

pub async fn run() -> std::io::Result<()> {
    let address = get_config().webserver.bind_address.as_str();
    let port = get_config().webserver.port;

    HttpServer::new(|| {
        App::new().service(index).service(
            web::scope("/api")
                .route("getPrice", web::post().to(get_price::handle))
                .route("getRegistration", web::post().to(get_registration::handle))
                .route("getPlainName", web::post().to(get_plain_name::handle))
                .route("estimateReceiveNameGas", web::post().to(name::handle))
                .route(
                    "estimateExpirationInfoGas",
                    web::post().to(expiration_info::handle),
                )
                .route("estimateRenewGas", web::post().to(renew_request::handle))
                .route(
                    "estimateRegisterGas",
                    web::post().to(register_request::handle),
                )
                .route("genImg", web::post().to(img_gen::handle)),
        )
    })
    .bind((address, port))?
    .run()
    .await
}

#[get("/")]
async fn index(_req: HttpRequest) -> impl Responder {
    "Index page"
}
