use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::Command;
use clap::{Parser, Subcommand};
use walkdir::WalkDir;
use anyhow::{Result, anyhow};
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(name = "rustvector")]
#[command(version = "0.6.0")]
#[command(author = "Satyaa & Clawdy")]
#[command(about = "🦀 RustVector: High-performance semantic brain", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add an entry (auto-embeds text)
    Add { content: String, metadata: String },
    /// Ingest a folder recursively (auto-converts non-MD via markitdown)
    Ingest { path: String },
    /// Semantic search (text-based)
    Search { query: String, #[arg(default_value_t = 5)] limit: usize },
    /// View stats
    Stats,
    /// Wipe the brain
    Purge,
    /// Quick configuration
    Config { 
        #[arg(short, long)] provider: Option<String>,
        #[arg(short, long)] model: Option<String>,
        #[arg(short, long)] key: Option<String>
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
            model: "llama3.2:1b".into(),
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
    fs::write(format!("{}/config.json", dir), serde_json::to_string_pretty(config)?)?;
    Ok(())
}

fn get_embedding(text: &str, config: &AppConfig) -> Result<Vec<f32>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    match config.provider.as_str() {
        "ollama" => {
            let res = client.post("http://localhost:11434/api/embeddings")
                .json(&serde_json::json!({ "model": config.model, "prompt": text }))
                .send().map_err(|e| anyhow!("Ollama Offline? {}", e))?;
            let json: serde_json::Value = res.json()?;
            let emb = json["embedding"].as_array()
                .ok_or_else(|| anyhow!("Ollama error: {:?}", json))?
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
                .ok_or_else(|| anyhow!("OpenAI error"))?.iter().map(|v| v.as_f64().unwrap() as f32).collect();
            Ok(emb)
        },
        _ => Err(anyhow!("Provider unsupported")),
    }
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
            println!("✅ Config saved to ~/.rustvector/config.json");
        }
        Commands::Add { content, metadata } => {
            let emb = get_embedding(content, &config)?;
            let mut bytes = Vec::with_capacity(emb.len() * 4);
            for f in emb { bytes.extend_from_slice(&f.to_le_bytes()); }
            conn.execute("INSERT INTO vectors (content, metadata, embedding, timestamp) VALUES (?1, ?2, ?3, ?4)", params![content, metadata, bytes, Utc::now().to_rfc3339()])?;
            println!("✅ Indexed.");
        }
        Commands::Ingest { path } => {
            let files: Vec<_> = WalkDir::new(path).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).collect();
            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
                .progress_chars("#>-"));

            for entry in files {
                let p = entry.path();
                pb.set_message(format!("Indexing: {}", entry.file_name().to_string_lossy()));
                
                let content = if p.extension().map_or(false, |ext| ext == "md" || ext == "txt") {
                    fs::read_to_string(p).ok()
                } else {
                    let output = Command::new("markitdown").arg(p).output();
                    output.ok().and_then(|o| String::from_utf8(o.stdout).ok())
                };

                if let Some(txt) = content {
                    if txt.len() > 0 && txt.len() < 200000 {
                        if let Ok(emb) = get_embedding(&txt, &config) {
                            let mut bytes = Vec::with_capacity(emb.len() * 4);
                            for f in emb { bytes.extend_from_slice(&f.to_le_bytes()); }
                            let _ = conn.execute("INSERT INTO vectors (content, metadata, embedding, timestamp) VALUES (?1, ?2, ?3, ?4)", 
                                params![txt, format!("{{\"path\": \"{}\"}}", p.display()), bytes, Utc::now().to_rfc3339()]);
                        }
                    }
                }
                pb.inc(1);
            }
            pb.finish_with_message("✅ Ingestion complete");
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
            for (content, meta, score, _) in results.iter().take(*limit) {
                println!("[{:.2}%] {}\n{}\n", score * 100.0, meta, &content[..content.len().min(120)].replace('\n', " "));
            }
        }
        Commands::Stats => {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM vectors", [], |r| r.get(0))?;
            println!("📊 Brain: {} vectors | Provider: {} | Model: {}", count, config.provider, config.model);
        }
        Commands::Purge => { conn.execute("DELETE FROM vectors", [])?; println!("🗑️ Wiped."); }
    }
    Ok(())
}
