mod discord;
mod scraper;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use anyhow::Result;
use indexmap::IndexSet; 
use reqwest::Client;
use std::env;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: cargo run -- \"search term\" [interval_minutes] (defaults to 30)");
    }

    let search_term = &args[1];
    let interval_mins: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(30);
    let webhook = env::var("DISCORD_WEBHOOK_URL").expect("Missing Webhook URL");
    let url = format!(
        "https://www.vinted.be/catalog?search_text={}&order=newest_first",
        search_term
    );

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let mut timer = interval(Duration::from_secs(interval_mins * 60));
    let mut seen_urls = IndexSet::with_capacity(100);

    println!("Monitoring: \"{}\"", search_term);

    loop {
        timer.tick().await;
        if let Err(e) = run_task(&client, &url, &webhook, &mut seen_urls).await {
            eprintln!("Error: {}", e);
        }
    }
}

async fn run_task(
    client: &Client,
    url: &str,
    webhook: &str,
    seen_urls: &mut IndexSet<String>,
) -> Result<()> {
    let res = scraper::fetch_html(client, url).await?;
    let tags = scraper::parse_html(&res).await?;

    if tags.is_empty() {
        anyhow::bail!("Found no items. Possible rate limit.");
    }

    let mut new_items = Vec::new();
    for (desc, item_url) in tags {
        if seen_urls.contains(&item_url) {
            break;
        }
        new_items.push((desc, item_url));
    }

    if new_items.is_empty() {
        println!("No new items.");
        return Ok(());
    }

    new_items.reverse();

    let mut buffer = String::with_capacity(2000);
    for (desc, item_url) in new_items {
        let clean_desc = discord::clean_description(&desc);
        let entry = format!("**{}**\n<{}> ||@here||\n\n", clean_desc.trim(), item_url);

        if buffer.len() + entry.len() > 1900 {
            discord::post_to_discord(client, &buffer, webhook).await?;
            buffer.clear();
        }
        buffer.push_str(&entry);
        seen_urls.insert(item_url);
    }

    while seen_urls.len() > 100 {
        seen_urls.shift_remove_index(0);
    }

    if !buffer.is_empty() {
        discord::post_to_discord(client, &buffer, webhook).await?;
    }

    println!("Updated (Cache size: {}).", seen_urls.len());
    Ok(())
}
