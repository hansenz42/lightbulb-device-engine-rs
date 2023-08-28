mod http_server;
use http_server::server::run;

fn main() {
    run().unwrap();
    println!("Hello, world!");
}