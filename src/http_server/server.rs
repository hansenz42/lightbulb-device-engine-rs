use actix::{get, web, App, HttpServer, Responsder};

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

// run server as a submodule
pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind("0.0.0.0")
        .unwrap();
}