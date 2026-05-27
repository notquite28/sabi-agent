//! Web search and code search via Exa direct API.
//!
//! Ported from:
//! - `pi-web-access/exa.ts`
//! - `pi-web-access/code-search.ts`
//!
//! Simplifications:
//! - Only direct Exa API. No MCP proxy, no Perplexity/Gemini fallback chain.
//! - Requires `exa_api_key` in `~/.sabi/auth.toml` or `EXA_API_KEY` in the environment.
//! - No budget tracking or activity monitor.

use anyhow::{Context, Result};

use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config;
use crate::tools::{object_schema, ToolOutput, ToolSpec};

const EXA_SEARCH_URL: &str = "https://api.exa.ai/search";
const EXA_ANSWER_URL: &str = "https://api.exa.ai/answer";

pub fn web_search_spec() -> ToolSpec {
    ToolSpec {
        name: "web_search",
        description: "Search the web for information. Returns results with sources and snippets. Requires exa_api_key in ~/.sabi/auth.toml or EXA_API_KEY in the environment.",
        parameters: object_schema(
            json!({
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "num_results": {
                    "type": "integer",
                    "description": "Number of results (default 5, max 20)",
                    "minimum": 1,
                    "maximum": 20
                }
            }),
            vec!["query"],
        ),
    }
}

pub fn exa_search_spec() -> ToolSpec {
    ToolSpec {
        name: "exa_search",
        description: "Search for code examples, documentation, and API references using Exa's neural code search. Requires exa_api_key in ~/.sabi/auth.toml or EXA_API_KEY in the environment.",
        parameters: object_schema(
            json!({
                "query": {
                    "type": "string",
                    "description": "Programming question, API, library, or debugging topic"
                },
                "num_results": {
                    "type": "integer",
                    "description": "Number of results (default 10, max 20)",
                    "minimum": 1,
                    "maximum": 20
                }
            }),
            vec!["query"],
        ),
    }
}

pub async fn run_web_search(args: Value) -> Result<ToolOutput> {
    let query = args["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing query"))?;
    let num_results = args["num_results"].as_u64().unwrap_or(5) as usize;

    let api_key = exa_api_key()?;

    // web_search always uses /search so that `num_results` is honoured.
    // /answer does not accept a result-count parameter.
    match search_with_exa_direct(query, num_results, &api_key).await {
        Ok(text) => Ok(ToolOutput {
            content: text,
            is_error: false,
            events: Vec::new(),
        }),
        Err(error) => Ok(ToolOutput {
            content: format!("Search error: {error}"),
            is_error: true,
            events: Vec::new(),
        }),
    }
}

pub async fn run_exa_search(args: Value) -> Result<ToolOutput> {
    let query = args["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing query"))?;
    let num_results = args["num_results"].as_u64().unwrap_or(10) as usize;

    let api_key = exa_api_key()?;

    // Use Exa search with code-focused query.
    let enriched_query = build_code_query(query);

    match search_with_exa(&enriched_query, num_results, &api_key).await {
        Ok(text) => Ok(ToolOutput {
            content: text,
            is_error: false,
            events: Vec::new(),
        }),
        Err(error) => Ok(ToolOutput {
            content: format!("Search error: {error}"),
            is_error: true,
            events: Vec::new(),
        }),
    }
}

fn exa_api_key() -> Result<String> {
    config::exa_api_key().context(
        "EXA_API_KEY not found. Add exa_api_key to ~/.sabi/auth.toml or set it in your environment",
    )
}

/// Try /answer first (synthesised answer) and fall back to /search.
async fn search_with_exa(query: &str, num_results: usize, api_key: &str) -> Result<String> {
    let client = Client::new();

    // Try /answer first for a synthesized answer.
    let answer_res = client
        .post(EXA_ANSWER_URL)
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&json!({
            "query": query,
            "text": true,
        }))
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await;

    if let Ok(response) = answer_res {
        if response.status().is_success() {
            let data: ExaAnswerResponse = response.json().await?;
            if let Some(answer) = data.answer {
                let citations = format_citations(&data.citations);
                return Ok(format!("{answer}\n\n{citations}"));
            }
        }
    }

    // Fall back to /search for raw results.
    search_with_exa_direct(query, num_results, api_key).await
}

/// Go straight to /search so that `numResults` is always honoured.
async fn search_with_exa_direct(query: &str, num_results: usize, api_key: &str) -> Result<String> {
    let client = Client::new();

    let response = client
        .post(EXA_SEARCH_URL)
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&json!({
            "query": query,
            "type": "auto",
            "numResults": num_results,
            "contents": {
                "text": { "maxCharacters": 3000 },
                "highlights": true,
            },
        }))
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .context("Exa API request failed")?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Exa API error: {error_text}");
    }

    let data: ExaSearchResponse = response.json().await?;
    Ok(format_search_results(&data.results))
}

fn format_search_results(results: &[ExaSearchResult]) -> String {
    if results.is_empty() {
        return "No results found.".to_string();
    }

    let mut parts = Vec::new();
    for (i, result) in results.iter().enumerate() {
        let title = result.title.as_deref().unwrap_or("Untitled");
        let url = result.url.as_deref().unwrap_or("");
        let highlights: Vec<_> = result
            .highlights
            .iter()
            .filter_map(|h| h.as_str())
            .collect();
        let content = if !highlights.is_empty() {
            highlights.join(" ")
        } else {
            result
                .text
                .as_deref()
                .unwrap_or("")
                .trim()
                .chars()
                .take(1000)
                .collect()
        };

        parts.push(format!(
            "{}. {}\n   URL: {}\n   {}\n",
            i + 1,
            title,
            url,
            content
        ));
    }

    parts.join("\n")
}

fn format_citations(citations: &Option<Vec<ExaCitation>>) -> String {
    let citations = match citations {
        Some(c) if !c.is_empty() => c,
        _ => return String::new(),
    };

    let mut lines = vec!["Sources:".to_string()];
    for (i, citation) in citations.iter().enumerate() {
        let title = citation.title.as_deref().unwrap_or("Untitled");
        let url = citation.url.as_deref().unwrap_or("");
        lines.push(format!("{}. {} ({})", i + 1, title, url));
    }
    lines.join("\n")
}

fn build_code_query(query: &str) -> String {
    let normalized = query.to_lowercase();
    let has_code_terms = [
        "api",
        "code",
        "docs",
        "documentation",
        "example",
        "github",
        "implementation",
        "library",
        "source",
        "stackoverflow",
    ]
    .iter()
    .any(|term| normalized.contains(term));

    if has_code_terms {
        query.to_string()
    } else {
        format!("{query} code examples documentation GitHub")
    }
}

// --- API response types ---

#[derive(Debug, Deserialize)]
struct ExaAnswerResponse {
    answer: Option<String>,
    citations: Option<Vec<ExaCitation>>,
}

#[derive(Debug, Deserialize)]
struct ExaCitation {
    url: Option<String>,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaSearchResult>,
}

#[derive(Debug, Deserialize)]
struct ExaSearchResult {
    title: Option<String>,
    url: Option<String>,
    text: Option<String>,
    highlights: Vec<Value>,
}
