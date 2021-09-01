use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};


use journal::query_journal;
use serde::Deserialize;

mod journal;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(list)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


#[derive(Deserialize)]
struct ListQuery {
    n : Option<usize>,
    severity : Option<String>,
    last_secs : Option<usize>,
    cursor : Option<String>,
}

#[get("/logs")]
async fn list(query: web::Query<ListQuery>) -> impl Responder {
    let logs = query_journal(&query.n, &query.severity, &query.last_secs, &query.cursor);

    match logs {
        Ok(entries) => {
            HttpResponse::Ok().json(entries)
        },
        Err(err) => {
            HttpResponse::BadRequest().body(err)
        },
    }
}