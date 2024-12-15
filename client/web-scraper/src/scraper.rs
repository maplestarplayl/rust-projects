use reqwest;
use scraper::{Html, Selector};
use anyhow::Result;
use crate::models::Article;
pub async fn scrape_articles(_url: &str) -> Result<Vec<String>> {
    let response = reqwest::get("https://www.rust-lang.org/learn").await?.text().await?;
    let document = Html::parse_document(&response);
    // let selector = Selector::parse("article").unwrap();
    let name = extract_book_name(document).unwrap();
    println!("{:?}", name);
    Ok(name)
}

fn extract_book_name(html: Html) -> Option<Vec<String>> {
    let selector = Selector::parse("a.button-secondary").unwrap();
    // html.select(&selector).next()?.text().collect::<String>().into()
    html.select(&selector).map(|e| e.text().collect::<String>()).collect::<Vec<String>>().into()
}
