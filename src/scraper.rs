use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

pub async fn fetch_html(client: &Client, url: &str) -> Result<String> {
    let body = client.get(url).send().await?.text().await?;
    Ok(body)
}

pub async fn parse_html(html: &str) -> Result<Vec<(String, String)>> {
    let doc = Html::parse_document(html);
    let item_selector = Selector::parse("a.new-item-box__overlay").unwrap();

    let results: Vec<(String, String)> = doc
        .select(&item_selector)
        .filter_map(|anchor| {
            let description = anchor.value().attr("title")?;
            let href = anchor.value().attr("href")?;
            Some((description.to_string(), href.to_string()))
        })
        .collect();

    Ok(results)
}
