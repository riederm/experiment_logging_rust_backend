use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use journal::read_last_n_entries;


mod journal;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    let logs = read_last_n_entries(10);
    if let Ok(entry) = logs {
        let message = format!("{:#?}", entry);
        //let message = entry.get_message().unwrap_or("none").to_string();
        HttpResponse::Ok().body(message)

    }else {
        HttpResponse::Ok().body("no entry :-(")
    }
}