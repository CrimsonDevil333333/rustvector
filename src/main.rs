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
#[command(version = "0.8.1")]
#[command(author = "Satyaa & Clawdy")]
#[command(about = "🦀 RustVector: High-performance semantic brain with delta indexing", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add an entry (auto-embeds text)
    Add { content: String, metadata: String },
    /// Ingest a folder recursively (Delta mode: skips unchanged files)
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
    /// List all stored vectors
    Ls { 
        #[arg(short, long, default_value_t = 10)] limit: i64,
        #[arg(short, long, default_value_t = 0)] offset: i64 
    },
    /// Remove a specific vector by ID
    Rm { id: i32 },
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
                .send().map_err(|e| anyhow!("Ollama connection error: {}", e))?;
            let json: serde_json::Value = res.json()?;
            let emb = json["embedding"].as_array()
                .ok_or_else(|| anyhow!("Ollama model error: model not found?"))?
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
                .ok_or_else(|| anyhow!("OpenAI API error"))?.iter().map(|v| v.as_f64().unwrap() as f32).collect();
            Ok(emb)
        },
        _ => Err(anyhow!("Provider not supported")),
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
    let db_dir = format!("{}/.rustvector", home);
    fs::create_dir_all(&db_dir)?;
    let db_path = format!("{}/vector.db", db_dir);
    let conn = Connection::open(&db_path)?;

    // Ensure schema is up to date
    conn.execute("CREATE TABLE IF NOT EXISTS vectors (
        id INTEGER PRIMARY KEY, 
        content TEXT NOT NULL, 
        metadata TEXT, 
        embedding BLOB NOT NULL, 
        timestamp TEXT NOT NULL
    )", [])?;

    // Delta Migration: Check if file_hash exists
    let has_hash_col: bool = conn.prepare("PRAGMA table_info(vectors)")?
        .query_map([], |row| Ok(row.get::<_, String>(1)?))?
        .any(|name| name.map_or(false, |n| n == "file_hash"));

    if !has_hash_col {
        conn.execute("ALTER TABLE vectors ADD COLUMN file_hash TEXT", [])?;
    }

    match &cli.command {
        Commands::Config { provider, model, key } => {
            let mut new_config = config.clone();
            if let Some(p) = provider { new_config.provider = p.clone(); }
            if let Some(m) = model { new_config.model = m.clone(); }
            if let Some(k) = key { new_config.api_key = Some(k.clone()); }
            save_config(&new_config)?;
            println!("✅ Config updated.");
        }
        Commands::Add { content, metadata, .. } => {
            let emb = get_embedding(content, &config)?;
            let mut bytes = Vec::with_capacity(emb.len() * 4);
            for f in emb { bytes.extend_from_slice(&f.to_le_bytes()); }
            conn.execute("INSERT INTO vectors (content, metadata, embedding, timestamp) VALUES (?1, ?2, ?3, ?4)", params![content, metadata, bytes, Utc::now().to_rfc3339()])?;
            println!("✅ Fact indexed.");
        }
        Commands::Ingest { path } => {
            let files: Vec<_> = WalkDir::new(path).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).collect();
            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
                .progress_chars("#>-"));

            let mut indexed = 0;
            let mut skipped = 0;

            for entry in files {
                let p = entry.path();
                let file_name = entry.file_name().to_string_lossy().into_owned();
                pb.set_message(format!("Checking: {}", file_name));
                
                let file_meta = fs::metadata(p)?;
                let current_hash = format!("{}-{:?}", file_meta.len(), file_meta.modified()?);
                let path_str = p.to_string_lossy().into_owned();
                
                let already_exists: bool = conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM vectors WHERE file_hash = ?1 AND metadata LIKE ?2)",
                    params![current_hash, format!("%{}%", path_str)],
                    |row| row.get(0)
                ).unwrap_or(false);

                if already_exists {
                    skipped += 1;
                    pb.inc(1);
                    continue;
                }

                let content = if p.extension().map_or(false, |ext| ext == "md" || ext == "txt") {
                    fs::read_to_string(p).ok()
                } else {
                    Command::new("markitdown").arg(p).output().ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                };

                if let Some(txt) = content {
                    if !txt.is_empty() && txt.len() < 250000 {
                        if let Ok(emb) = get_embedding(&txt, &config) {
                            let mut b = Vec::with_capacity(emb.len() * 4);
                            for f in emb { b.extend_from_slice(&f.to_le_bytes()); }
                            conn.execute("DELETE FROM vectors WHERE metadata LIKE ?1", params![format!("%{}%", path_str)])?;
                            conn.execute(
                                "INSERT INTO vectors (content, metadata, embedding, timestamp, file_hash) VALUES (?1, ?2, ?3, ?4, ?5)", 
                                params![txt, format!("{{\"path\": \"{}\"}}", path_str), b, Utc::now().to_rfc3339(), current_hash]
                            )?;
                            indexed += 1;
                        }
                    }
                }
                pb.inc(1);
            }
            pb.finish_with_message(format!("✅ Done. Indexed: {}, Skipped: {}", indexed, skipped));
        }
        Commands::Search { query, limit } => {
            let q_vec = get_embedding(query, &config)?;
            let mut stmt = conn.prepare("SELECT content, metadata, embedding FROM vectors")?;
            let rows = stmt.query_map([], |row| {
                let bytes: Vec<u8> = row.get(2)?;
                let emb: Vec<f32> = bytes.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
                let content: String = row.get(0)?;
                let meta: String = row.get(1)?;
                Ok((content, meta, emb))
            })?;
            let mut results = Vec::new();
            for r in rows {
                let (c, m, e) = r?;
                results.push((c, m, cosine_similarity(&q_vec, &e)));
            }
            results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            for (c, m, s) in results.iter().take(*limit) {
                let preview = if c.len() > 120 { &c[..120] } else { &c };
                println!("[{:.2}% match] {}\nPreview: {}\n", s * 100.0, m, preview.replace('\n', " "));
            }
        }
        Commands::Stats => {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM vectors", [], |r| r.get(0))?;
            println!("📊 RustVector Stats");
            println!("Total vectors: {}", count);
            println!("Provider:      {}", config.provider);
            println!("Model:         {}", config.model);
        }
        Commands::Purge => {
            conn.execute("DELETE FROM vectors", [])?;
            println!("🗑️ Brain wiped.");
        }
        Commands::Ls { limit, offset } => {
            let mut stmt = conn.prepare("SELECT id, metadata, timestamp, substr(content, 1, 50) FROM vectors LIMIT ?1 OFFSET ?2")?;
            let rows = stmt.query_map(params![limit, offset], |row| {
                Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))
            })?;
            println!("{:<5} | {:<20} | {:<25} | {:<50}", "ID", "Timestamp", "Metadata", "Snippet");
            println!("{}", "-".repeat(110));
            for r in rows {
                let (id, meta, ts, snip) = r?;
                println!("{:<5} | {:<20} | {:<25} | {:<50}", id, &ts[..ts.len().min(19)], meta, snip.replace('\n', " "));
            }
        }
        Commands::Rm { id } => {
            let deleted = conn.execute("DELETE FROM vectors WHERE id = ?1", params![id])?;
            if deleted > 0 { println!("✅ Removed ID: {}", id); }
            else { println!("❌ ID not found."); }
        }
    }
    Ok(())
}
