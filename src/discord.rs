use aho_corasick::{AhoCorasick, MatchKind};
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::OnceLock;

pub async fn post_to_discord(client: &Client, msg: &str, webhook: &str) -> Result<String> {
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("content", msg);

    let res = client.post(webhook).json(&map).send().await?.text().await?;
    Ok(res)
}

pub fn clean_description(desc: &str) -> String {
    static AC: OnceLock<AhoCorasick> = OnceLock::new();
    let patterns = &[
        " Protection acheteurs incluse",
        " Protection acheteurs (Pro) incluse",
        "état: ",
        "taille: ",
        "marque:",
        "Comme neuf",
        "Neuf",
        "Très bon état",
        "Bon état",
        "Satisfaisant",
        "sans étiquette",
    ];
    let replace_with = &[
        "",
        "",
        "\n ",
        "Size: ",
        "Brand:",
        "Like new",
        "New",
        "Very good",
        "Good",
        "Satisfactory",
        "without etiquette",
    ];
    let ac = AC.get_or_init(|| {
        AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostFirst)
            .build(patterns)
            .unwrap()
    });
    ac.replace_all(desc, replace_with)
}
