use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use clap::{Parser, Subcommand};
use walkdir::WalkDir;
use anyhow::{Result, anyhow};
use chrono::Utc;

#[derive(Parser)]
#[command(name = "rustvector")]
#[command(version = "0.5.0")]
#[command(author = "Clawdy <clawdy@openclaw.ai>")]
#[command(about = "🦀 RustVector: Ultra-lightweight Semantic Brain for Edge Devices", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add an entry (auto-embeds text)
    Add { content: String, metadata: String },
    /// Ingest a folder recursively (auto-embeds every file)
    Ingest { path: String },
    /// Semantic search (just pass natural language text)
    Search { query: String, #[arg(default_value_t = 5)] limit: usize },
    /// View total vectors and current provider config
    Stats,
    /// Wipe the entire brain database
    Purge,
    /// Configure embedding provider (ollama, openai, gemini)
    Config { 
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(short, long)]
        model: Option<String>,
        #[arg(short, long)]
        key: Option<String>
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppConfig {
    provider: String,
    model: String,
    api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".into(),
            model: "all-minilm".into(),
            api_key: None,
        }
    }
}

fn load_config() -> AppConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let path = format!("{}/.rustvector/config.json", home);
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

fn save_config(config: &AppConfig) -> Result<()> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let dir = format!("{}/.rustvector", home);
    fs::create_dir_all(&dir)?;
    let data = serde_json::to_string_pretty(config)?;
    fs::write(format!("{}/config.json", dir), data)?;
    Ok(())
}

fn get_embedding(text: &str, config: &AppConfig) -> Result<Vec<f32>> {
    let client = reqwest::blocking::Client::new();
    
    match config.provider.as_str() {
        "ollama" => {
            let res = client.post("http://localhost:11434/api/embeddings")
                .json(&serde_json::json!({ "model": config.model, "prompt": text }))
                .send()?;
            let json: serde_json::Value = res.json()?;
            let emb = json["embedding"].as_array()
                .ok_or_else(|| anyhow!("Ollama error: Check if model '{}' is pulled", config.model))?
                .iter().map(|v| v.as_f64().unwrap() as f32).collect();
            Ok(emb)
        },
        "openai" => {
            let key = config.api_key.as_ref().ok_or_else(|| anyhow!("OpenAI key missing"))?;
            let res = client.post("https://api.openai.com/v1/embeddings")
                .header("Authorization", format!("Bearer {}", key))
                .json(&serde_json::json!({ "model": config.model, "input": text }))
                .send()?;
            let json: serde_json::Value = res.json()?;
            let emb = json["data"][0]["embedding"].as_array()
                .ok_or_else(|| anyhow!("OpenAI error: {:?}", json))?
                .iter().map(|v| v.as_f64().unwrap() as f32).collect();
            Ok(emb)
        },
        "gemini" => {
            let key = config.api_key.as_ref().ok_or_else(|| anyhow!("Gemini key missing"))?;
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}", config.model, key);
            let res = client.post(url)
                .json(&serde_json::json!({ "content": { "parts": [{ "text": text }] } }))
                .send()?;
            let json: serde_json::Value = res.json()?;
            let emb = json["embedding"]["values"].as_array()
                .ok_or_else(|| anyhow!("Gemini error: {:?}", json))?
                .iter().map(|v| v.as_f64().unwrap() as f32).collect();
            Ok(emb)
        },
        _ => Err(anyhow!("Provider '{}' not supported", config.provider)),
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() { return 0.0; }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
    dot / (norm_a * norm_b)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = load_config();
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let db_path = format!("{}/.rustvector/vector.db", home);
    fs::create_dir_all(format!("{}/.rustvector", home))?;
    let conn = Connection::open(&db_path)?;

    conn.execute("CREATE TABLE IF NOT EXISTS vectors (id INTEGER PRIMARY KEY, content TEXT NOT NULL, metadata TEXT, embedding BLOB NOT NULL, timestamp TEXT NOT NULL)", [])?;

    match &cli.command {
        Commands::Config { provider, model, key } => {
            let mut new_config = config.clone();
            if let Some(p) = provider { new_config.provider = p.clone(); }
            if let Some(m) = model { new_config.model = m.clone(); }
            if let Some(k) = key { new_config.api_key = Some(k.clone()); }
            save_config(&new_config)?;
            println!("✅ Config updated: {} using model {}", new_config.provider, new_config.model);
        }
        Commands::Add { content, metadata } => {
            let emb = get_embedding(content, &config)?;
            let mut bytes = Vec::with_capacity(emb.len() * 4);
            for f in emb { bytes.extend_from_slice(&f.to_le_bytes()); }
            conn.execute("INSERT INTO vectors (content, metadata, embedding, timestamp) VALUES (?1, ?2, ?3, ?4)", params![content, metadata, bytes, Utc::now().to_rfc3339()])?;
            println!("✅ Content indexed via {}.", config.provider);
        }
        Commands::Ingest { path } => {
            let mut count = 0;
            println!("🧠 Ingesting: {}", path);
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.len() > 100000 || content.contains('\0') { continue; }
                        if let Ok(emb) = get_embedding(&content, &config) {
                            let mut bytes = Vec::with_capacity(emb.len() * 4);
                            for f in emb { bytes.extend_from_slice(&f.to_le_bytes()); }
                            let meta = format!("{{\"path\": \"{}\"}}", entry.path().display());
                            conn.execute("INSERT INTO vectors (content, metadata, embedding, timestamp) VALUES (?1, ?2, ?3, ?4)", params![content, meta, bytes, Utc::now().to_rfc3339()])?;
                            count += 1;
                            println!("  + {}", entry.file_name().to_string_lossy());
                        }
                    }
                }
            }
            println!("✅ Successfully indexed {} files.", count);
        }
        Commands::Search { query, limit } => {
            let query_vec = get_embedding(query, &config)?;
            let mut stmt = conn.prepare("SELECT content, metadata, embedding, timestamp FROM vectors")?;
            let rows = stmt.query_map([], |row| {
                let bytes: Vec<u8> = row.get(2)?;
                let embedding: Vec<f32> = bytes.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, embedding, row.get::<_, String>(3)?))
            })?;
            let mut results = Vec::new();
            for row in rows {
                let (content, meta, emb, ts) = row?;
                results.push((content, meta, cosine_similarity(&query_vec, &emb), ts));
            }
            results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            println!("--- Top {} Matches (via {}) ---", limit, config.provider);
            for (content, meta, score, ts) in results.iter().take(*limit) {
                println!("[Relevance: {:.2}%] [At: {}]\nMeta: {}\nContent: {}...\n", score * 100.0, ts, meta, &content[..content.len().min(150)].replace('\n', " "));
            }
        }
        Commands::Stats => {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM vectors", [], |r| r.get(0))?;
            println!("📊 RustVector Statistics");
            println!("Total Vectors: {}", count);
            println!("Provider:      {}", config.provider);
            println!("Model:         {}", config.model);
            println!("DB Path:       {}/.rustvector/vector.db", home);
        }
        Commands::Purge => {
            conn.execute("DELETE FROM vectors", [])?;
            println!("🗑️ Brain wiped.");
        }
    }
    Ok(())
}
