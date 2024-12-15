mod models;
mod scraper;
use actix_web::{HttpServer, App, web};
use actix_web::{HttpResponse, Responder};
use serde_json::json;
use scraper::scrape_articles;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_articles))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
async fn get_articles() -> impl Responder {
    match scrape_articles("https://example.com").await {
        Ok(articles) => HttpResponse::Ok().json(json!({ "articles": articles })),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve articles"),
    }
}
