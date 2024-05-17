use std::error::Error;

use actix_web::{get, web, App, HttpServer, Responder};
use crate::common::setting::Settings;

async fn index() -> impl Responder {
    "Hello, World!"
}

pub struct CustomHttpServer {}

impl CustomHttpServer {
    pub async fn start() ->  Result<(), Box<dyn Error>>{
        let server = HttpServer::new(|| {
            App::new().service(
                // prefixes all resources and routes attached to it...
                web::scope("/app")
                    // ...so this handles requests for `GET /app/index.html`
                    .route("/index.html", web::get().to(index)),
            )
        })
        .bind((Settings::get().web.web_host.as_str(), Settings::get().web.web_port))?
        .workers(2)
        .run();
    
        tokio::spawn(server);

        log::info!("http server started host: {} port: {}", Settings::get().web.web_host, Settings::get().web.web_port);
    
        Ok(())
    }
}