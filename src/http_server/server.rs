use actix_web::{get, web, App, HttpServer, Responder};
use crate::common::setting::Settings;

async fn index() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
// run server as a submodule
pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            // prefixes all resources and routes attached to it...
            web::scope("/app")
                // ...so this handles requests for `GET /app/index.html`
                .route("/index.html", web::get().to(index)),
        )
    })
    .bind((Settings::get().web.web_host.as_str(), Settings::get().web.web_port))?
    .run()
    .await?;

    Ok(())
}