use actix_web::{get, App, HttpResponse, HttpServer};
use actix_web::{HttpRequest, Responder};
use std::fs;
use std::time::{Duration, SystemTime};

extern crate markdown;

#[get("/")]
async fn root(req: HttpRequest) -> impl Responder {
    let con_info = req.connection_info();
    let peer_ip = &req.peer_addr().unwrap().to_string();

    let ip = con_info
        .realip_remote_addr()
        .unwrap()
        .split(':')
        .next()
        .unwrap_or(peer_ip);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0));

    println!(
        "[{}][{}][{}]: Processing cv request...",
        now.as_secs(),
        ip,
        peer_ip
    );

    let data = fs::read_to_string("./cv.md").expect("Unable to read file");
    let html: String = markdown::to_html(&data);

    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(root))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
