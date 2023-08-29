use actix_web::{get, web, App, HttpServer, Responder};

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
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}