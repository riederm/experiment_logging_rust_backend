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
}

#[get("/logs")]
async fn list(query: web::Query<ListQuery>) -> impl Responder {
    let logs = query_journal(&query.n, &query.severity);
    if let Ok(entry) = logs {
        HttpResponse::Ok().json(entry)
    } else {
        HttpResponse::Ok().body("no entry :-(")
    }
}