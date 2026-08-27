use std::{net::IpAddr, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use chrono::{Duration as ChronoDuration, Utc};
use reqwest::{redirect::Policy, Client};
use scraper::{Html, Selector};
use similar::{ChangeTag, TextDiff};
use sqlx::SqlitePool;
use url::Url;
use uuid::Uuid;

use crate::models::{CheckResult, Source};

const USER_AGENT: &str = "ChangeDiffInbox/1.0 (+https://change-diff-inbox.sociobot.in)";
const MAX_BYTES: usize = 2_000_000;

pub async fn check_source(pool: &SqlitePool, id: &str) -> Result<CheckResult> {
    let source = sqlx::query_as::<_, Source>("SELECT * FROM sources WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Source not found"))?;

    let now = Utc::now();
    if let Some(last_checked) = source
        .last_checked
        .as_deref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
    {
        if now
            .signed_duration_since(last_checked.with_timezone(&Utc))
            .num_seconds()
            < 30
        {
            return Ok(CheckResult {
                outcome: "cooldown".into(),
                message: "This source was checked less than 30 seconds ago.".into(),
                change_id: None,
            });
        }
    }
    let next = now + ChronoDuration::minutes(source.interval_minutes);
    let result = fetch_extract(&source).await;
    match result {
        Ok(content) => {
            if content.trim().is_empty() {
                return record_error(pool, &source, "The selected content was empty", &now, &next)
                    .await;
            }
            let Some(previous) = source.baseline.as_deref() else {
                sqlx::query("UPDATE sources SET baseline=?, last_checked=?, last_status='ready', last_error=NULL, next_check=? WHERE id=?")
                    .bind(&content).bind(now.to_rfc3339()).bind(next.to_rfc3339()).bind(&source.id)
                    .execute(pool).await?;
                return Ok(CheckResult {
                    outcome: "baseline".into(),
                    message: "Baseline captured. Future checks will create semantic diffs.".into(),
                    change_id: None,
                });
            };

            let ratio = change_ratio(previous, &content);
            if ratio < source.threshold {
                sqlx::query("UPDATE sources SET baseline=?, last_checked=?, last_status='quiet', last_error=NULL, next_check=? WHERE id=?")
                    .bind(&content).bind(now.to_rfc3339()).bind(next.to_rfc3339()).bind(&source.id)
                    .execute(pool).await?;
                return Ok(CheckResult {
                    outcome: "noise".into(),
                    message: format!(
                        "Change {:.1}% stayed below the {:.1}% threshold.",
                        ratio * 100.0,
                        source.threshold * 100.0
                    ),
                    change_id: None,
                });
            }

            let change_id = Uuid::new_v4().to_string();
            let summary = summarize(previous, &content);
            let mut tx = pool.begin().await?;
            sqlx::query("INSERT INTO changes (id,source_id,previous_text,current_text,change_ratio,summary,created_at) VALUES (?,?,?,?,?,?,?)")
                .bind(&change_id).bind(&source.id).bind(previous).bind(&content).bind(ratio).bind(summary).bind(now.to_rfc3339())
                .execute(&mut *tx).await?;
            sqlx::query("UPDATE sources SET baseline=?, last_checked=?, last_status='changed', last_error=NULL, next_check=? WHERE id=?")
                .bind(&content).bind(now.to_rfc3339()).bind(next.to_rfc3339()).bind(&source.id)
                .execute(&mut *tx).await?;
            tx.commit().await?;
            Ok(CheckResult {
                outcome: "changed".into(),
                message: format!("Meaningful change detected ({:.1}%).", ratio * 100.0),
                change_id: Some(change_id),
            })
        }
        Err(error) => record_error(pool, &source, &error.to_string(), &now, &next).await,
    }
}

async fn record_error(
    pool: &SqlitePool,
    source: &Source,
    message: &str,
    now: &chrono::DateTime<Utc>,
    next: &chrono::DateTime<Utc>,
) -> Result<CheckResult> {
    sqlx::query("UPDATE sources SET last_checked=?, last_status='error', last_error=?, next_check=? WHERE id=?")
        .bind(now.to_rfc3339()).bind(message).bind(next.to_rfc3339()).bind(&source.id)
        .execute(pool).await?;
    Ok(CheckResult {
        outcome: "error".into(),
        message: message.into(),
        change_id: None,
    })
}

async fn fetch_extract(source: &Source) -> Result<String> {
    let url = validate_public_url(&source.url).await?;
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(20))
        .redirect(Policy::none())
        .build()?;
    ensure_robots_allowed(&client, &url).await?;

    let response = client
        .get(url.clone())
        .send()
        .await
        .context("Could not reach this source")?;
    if response.status().is_redirection() {
        bail!("Redirects are not followed automatically; save the destination URL instead")
    }
    if !response.status().is_success() {
        bail!("Source returned HTTP {}", response.status().as_u16())
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.contains("text/html") && !content_type.contains("application/xhtml") {
        bail!("Source is not an HTML page")
    }
    if response.content_length().unwrap_or(0) > MAX_BYTES as u64 {
        bail!("Source is larger than the 2 MB safety limit")
    }
    let bytes = response.bytes().await?;
    if bytes.len() > MAX_BYTES {
        bail!("Source is larger than the 2 MB safety limit")
    }
    let html = String::from_utf8_lossy(&bytes);
    let extracted = extract(&html, &source.selector, &source.extract_mode)?;
    if extracted.len() > 250_000 {
        bail!(
            "Selected content is larger than the 250 KB extraction limit; use a narrower selector"
        )
    }
    Ok(extracted)
}

async fn validate_public_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).context("Enter a valid absolute URL")?;
    if !matches!(url.scheme(), "http" | "https") {
        bail!("Only http and https URLs are supported")
    }
    if !url.username().is_empty() || url.password().is_some() {
        bail!("Authenticated URLs are not supported")
    }
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("URL must include a host"))?;
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".local") {
        bail!("Local network URLs are not allowed")
    }
    let addresses = tokio::net::lookup_host((host, url.port_or_known_default().unwrap_or(443)))
        .await
        .context("Could not resolve the source host")?;
    for address in addresses {
        if is_private(address.ip()) {
            bail!("Private or local network URLs are not allowed")
        }
    }
    Ok(url)
}

fn is_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_unspecified()
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

async fn ensure_robots_allowed(client: &Client, page: &Url) -> Result<()> {
    let mut robots_url = page.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);
    let response = match client.get(robots_url).send().await {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    if !response.status().is_success() {
        return Ok(());
    }
    let text = response.text().await.unwrap_or_default();
    let target = match page.query() {
        Some(query) => format!("{}?{query}", page.path()),
        None => page.path().to_owned(),
    };
    if robots_allows(&text, &target) {
        Ok(())
    } else {
        bail!("Blocked by this site's robots.txt")
    }
}

fn robots_allows(text: &str, target: &str) -> bool {
    let product = "changediffinbox";
    type RobotsRule = (bool, String);
    type RobotsGroup = (Vec<String>, Vec<RobotsRule>);

    let mut groups: Vec<RobotsGroup> = Vec::new();
    let mut agents = Vec::new();
    let mut rules = Vec::new();

    for raw in text.lines().chain(std::iter::once("")) {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            if !agents.is_empty() && !rules.is_empty() {
                groups.push((std::mem::take(&mut agents), std::mem::take(&mut rules)));
            }
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if key == "user-agent" {
            if !rules.is_empty() {
                groups.push((std::mem::take(&mut agents), std::mem::take(&mut rules)));
            }
            agents.push(value.to_ascii_lowercase());
        } else if (key == "allow" || key == "disallow") && !agents.is_empty() && !value.is_empty() {
            rules.push((key == "allow", value.to_owned()));
        }
    }

    let specificity = groups
        .iter()
        .flat_map(|(agents, _)| agents)
        .filter_map(|agent| {
            if agent == "*" {
                Some(0)
            } else if product.starts_with(agent) {
                Some(agent.len())
            } else {
                None
            }
        })
        .max();
    let Some(specificity) = specificity else {
        return true;
    };
    let mut matching: Vec<(bool, &str)> = groups
        .iter()
        .filter(|(agents, _)| {
            agents.iter().any(|agent| {
                (agent == "*" && specificity == 0)
                    || (agent != "*" && agent.len() == specificity && product.starts_with(agent))
            })
        })
        .flat_map(|(_, rules)| rules.iter().map(|(allow, path)| (*allow, path.as_str())))
        .filter(|(_, path)| target.starts_with(path.trim_end_matches('$')))
        .collect();
    matching.sort_by_key(|(allow, path)| (path.len(), *allow));
    matching.last().map(|(allow, _)| *allow).unwrap_or(true)
}

pub fn extract(html: &str, selector: &str, mode: &str) -> Result<String> {
    let document = Html::parse_document(html);
    let effective = match mode {
        "jsonld" => "script[type='application/ld+json']",
        "code" => {
            if selector.trim().is_empty() {
                "pre, code"
            } else {
                selector
            }
        }
        "table" => {
            if selector.trim().is_empty() {
                "table"
            } else {
                selector
            }
        }
        _ => {
            if selector.trim().is_empty() {
                "main"
            } else {
                selector
            }
        }
    };
    let parsed = Selector::parse(effective).map_err(|_| anyhow!("CSS selector is not valid"))?;
    let parts: Vec<String> = document
        .select(&parsed)
        .map(|node| {
            if mode == "jsonld" {
                node.inner_html()
            } else {
                node.text().collect::<Vec<_>>().join(" ")
            }
        })
        .collect();
    if parts.is_empty() {
        bail!("Selector matched no content")
    }
    Ok(normalize(&parts.join("\n")))
}

fn normalize(input: &str) -> String {
    input
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn change_ratio(old: &str, new: &str) -> f64 {
    if old == new {
        return 0.0;
    }
    let changes = TextDiff::from_words(old, new);
    let total = changes.iter_all_changes().count().max(1) as f64;
    let changed = changes
        .iter_all_changes()
        .filter(|c| c.tag() != ChangeTag::Equal)
        .count() as f64;
    changed / total
}

pub fn summarize(old: &str, new: &str) -> String {
    let diff = TextDiff::from_words(old, new);
    let added: Vec<&str> = diff
        .iter_all_changes()
        .filter(|c| c.tag() == ChangeTag::Insert)
        .map(|c| c.value().trim())
        .filter(|v| !v.is_empty())
        .take(10)
        .collect();
    let removed = diff
        .iter_all_changes()
        .filter(|c| c.tag() == ChangeTag::Delete)
        .count();
    if !added.is_empty() {
        format!(
            "Added {}{}",
            added.join(" "),
            if removed > 0 {
                format!(" · {removed} removal(s)")
            } else {
                String::new()
            }
        )
    } else if removed > 0 {
        format!("Removed {removed} segment(s)")
    } else {
        "Content changed".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracts_selected_table() {
        let html = "<table id='plans'><tr><td>Pro</td><td>$12</td></tr></table><p>ignore</p>";
        assert_eq!(extract(html, "#plans", "table").unwrap(), "Pro $12");
    }
    #[test]
    fn rejects_bad_selector() {
        assert!(extract("<main>x</main>", "[[", "selector").is_err());
    }
    #[test]
    fn change_threshold_math() {
        assert_eq!(change_ratio("same", "same"), 0.0);
        assert!(change_ratio("price is 10", "price is 12") > 0.0);
    }
    #[test]
    fn robots_uses_specific_group_and_longest_rule() {
        let robots = "User-agent: *\nDisallow: /private\nAllow: /private/status\n\nUser-agent: Changediffinbox\nUser-agent: AnotherBot\nDisallow: /internal";
        assert!(robots_allows(robots, "/private/status"));
        assert!(robots_allows(robots, "/private/other"));
        assert!(!robots_allows(robots, "/internal/build"));
    }
}
