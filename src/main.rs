use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use anyhow::Context;

#[derive(Serialize, Deserialize, Debug)]
struct VectorEntry {
    id: i32,
    content: String,
    metadata: String,
    embedding: Vec<u8>,
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Use a fixed path in home dir for global persistence, or local if specified
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let db_dir = format!("{}/.rustvector", home);
    fs::create_dir_all(&db_dir)?;
    let db_path = format!("{}/vector.db", db_dir);
    
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS vectors (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            metadata TEXT,
            embedding BLOB NOT NULL
        )",
        [],
    )?;

    if args.len() < 2 {
        println!("RustVector 🦀⚡");
        println!("Usage:");
        println!("  rustvector add <content> <metadata> <embeddings_csv>");
        println!("  rustvector ingest <folder_path> <embeddings_csv_for_all_files>");
        println!("  rustvector search <query_embeddings_csv>");
        return Ok(());
    }

    match args[1].as_str() {
        "add" => {
            let content = &args[2];
            let metadata = &args[3];
            let emb_str = &args[4];
            let emb: Vec<f32> = emb_str.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();
            let mut bytes = Vec::with_capacity(emb.len() * 4);
            for f in emb { bytes.extend_from_slice(&f.to_le_bytes()); }
            
            conn.execute(
                "INSERT INTO vectors (content, metadata, embedding) VALUES (?1, ?2, ?3)",
                params![content, metadata, bytes],
            )?;
            println!("✅ Added entry.");
        }
        "ingest" => {
            let path = Path::new(&args[2]);
            let emb_str = &args[3]; // Placeholder: real version would call an embedding API/model per file
            let emb: Vec<f32> = emb_str.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();
            let mut bytes = Vec::with_capacity(emb.len() * 4);
            for f in &emb { bytes.extend_from_slice(&f.to_le_bytes()); }

            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let content = fs::read_to_string(entry.path())?;
                    let meta = format!("{{\"path\": \"{}\"}}", entry.path().display());
                    conn.execute(
                        "INSERT INTO vectors (content, metadata, embedding) VALUES (?1, ?2, ?3)",
                        params![content, meta, bytes],
                    )?;
                    println!("📖 Ingested: {}", entry.file_name().to_string_lossy());
                }
            }
        }
        "search" => {
            let query_str = &args[2];
            let query_vec: Vec<f32> = query_str.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();

            let mut stmt = conn.prepare("SELECT content, metadata, embedding FROM vectors")?;
            let rows = stmt.query_map([], |row| {
                let bytes: Vec<u8> = row.get(2)?;
                let embedding: Vec<f32> = bytes.chunks_exact(4)
                    .map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, embedding))
            })?;

            let mut results = Vec::new();
            for row in rows {
                let (content, meta, emb) = row?;
                results.push((content, meta, cosine_similarity(&query_vec, &emb)));
            }
            results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
            
            for (content, meta, score) in results.iter().take(3) {
                println!("[{:.4}] Metadata: {}\nContent Preview: {}...\n", score, meta, &content[..content.len().min(100)]);
            }
        }
        _ => println!("Unknown command."),
    }

    Ok(())
}
