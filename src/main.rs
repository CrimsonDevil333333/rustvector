use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
<<<<<<< HEAD
use anyhow::Context;
=======
use chrono::Utc;
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)

#[derive(Serialize, Deserialize, Debug)]
struct VectorEntry {
    id: i32,
    content: String,
    metadata: String, 
    embedding: Vec<u8>,
    timestamp: String,
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
<<<<<<< HEAD
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
=======
    if a.len() != b.len() { return 0.0; }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
    dot / (norm_a * norm_b)
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
<<<<<<< HEAD
    
    // Use a fixed path in home dir for global persistence, or local if specified
=======
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let db_dir = format!("{}/.rustvector", home);
    fs::create_dir_all(&db_dir)?;
    let db_path = format!("{}/vector.db", db_dir);
    
<<<<<<< HEAD
    let conn = Connection::open(db_path)?;
=======
    let conn = Connection::open(&db_path)?;
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)

    conn.execute(
        "CREATE TABLE IF NOT EXISTS vectors (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            metadata TEXT,
            embedding BLOB NOT NULL,
            timestamp TEXT NOT NULL
        )",
        [],
    )?;

    if args.len() < 2 {
<<<<<<< HEAD
        println!("RustVector 🦀⚡");
        println!("Usage:");
        println!("  rustvector add <content> <metadata> <embeddings_csv>");
        println!("  rustvector ingest <folder_path> <embeddings_csv_for_all_files>");
        println!("  rustvector search <query_embeddings_csv>");
=======
        println!("🦀 RustVector CLI v0.2.0");
        println!("Standalone, light-weight, local vector brain.");
        println!("\nUsage:");
        println!("  rustvector add <content> <metadata_json> <emb_csv>");
        println!("  rustvector ingest <folder> <default_emb_csv>");
        println!("  rustvector search <query_emb_csv> [limit]");
        println!("  rustvector stats");
        println!("  rustvector export <output_json>");
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)
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
<<<<<<< HEAD
                "INSERT INTO vectors (content, metadata, embedding) VALUES (?1, ?2, ?3)",
                params![content, metadata, bytes],
            )?;
            println!("✅ Added entry.");
        }
        "ingest" => {
            let path = Path::new(&args[2]);
            let emb_str = &args[3]; // Placeholder: real version would call an embedding API/model per file
=======
                "INSERT INTO vectors (content, metadata, embedding, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![content, metadata, bytes, Utc::now().to_rfc3339()],
            )?;
            println!("✅ Content indexed.");
        }
        "ingest" => {
            let path = Path::new(&args[2]);
            let emb_str = &args[3];
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)
            let emb: Vec<f32> = emb_str.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();
            let mut bytes = Vec::with_capacity(emb.len() * 4);
            for f in &emb { bytes.extend_from_slice(&f.to_le_bytes()); }

            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
<<<<<<< HEAD
                    let content = fs::read_to_string(entry.path())?;
                    let meta = format!("{{\"path\": \"{}\"}}", entry.path().display());
                    conn.execute(
                        "INSERT INTO vectors (content, metadata, embedding) VALUES (?1, ?2, ?3)",
                        params![content, meta, bytes],
                    )?;
                    println!("📖 Ingested: {}", entry.file_name().to_string_lossy());
=======
                    let p = entry.path();
                    if p.is_file() {
                        if let Ok(content) = fs::read_to_string(&p) {
                            let meta = format!("{{\"path\": \"{}\", \"indexed_at\": \"{}\"}}", p.display(), Utc::now().to_rfc3339());
                            conn.execute(
                                "INSERT INTO vectors (content, metadata, embedding, timestamp) VALUES (?1, ?2, ?3, ?4)",
                                params![content, meta, bytes, Utc::now().to_rfc3339()],
                            )?;
                            println!("📖 Ingested: {}", entry.file_name().to_string_lossy());
                        }
                    }
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)
                }
            }
        }
        "search" => {
            let query_str = &args[2];
<<<<<<< HEAD
            let query_vec: Vec<f32> = query_str.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();

            let mut stmt = conn.prepare("SELECT content, metadata, embedding FROM vectors")?;
=======
            let limit: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
            let query_vec: Vec<f32> = query_str.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();

            let mut stmt = conn.prepare("SELECT content, metadata, embedding, timestamp FROM vectors")?;
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)
            let rows = stmt.query_map([], |row| {
                let bytes: Vec<u8> = row.get(2)?;
                let embedding: Vec<f32> = bytes.chunks_exact(4)
                    .map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
<<<<<<< HEAD
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, embedding))
=======
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, embedding, row.get::<_, String>(3)?))
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)
            })?;

            let mut results = Vec::new();
            for row in rows {
<<<<<<< HEAD
                let (content, meta, emb) = row?;
                results.push((content, meta, cosine_similarity(&query_vec, &emb)));
            }
            results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
            
            for (content, meta, score) in results.iter().take(3) {
                println!("[{:.4}] Metadata: {}\nContent Preview: {}...\n", score, meta, &content[..content.len().min(100)]);
            }
        }
=======
                let (content, meta, emb, ts) = row?;
                results.push((content, meta, cosine_similarity(&query_vec, &emb), ts));
            }
            results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
            
            println!("--- Top {} Results ---", limit);
            for (content, meta, score, ts) in results.iter().take(limit) {
                println!("[Score: {:.4}] [At: {}]\nMeta: {}\nPreview: {}...\n", score, ts, meta, &content[..content.len().min(120)].replace('\n', " "));
            }
        }
        "stats" => {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM vectors", [], |r| r.get(0))?;
            println!("📊 Total Vectors: {}", count);
            println!("📂 Location: {}", db_path);
        }
        "export" => {
            let out_path = &args[2];
            let mut stmt = conn.prepare("SELECT content, metadata, timestamp FROM vectors")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            })?;
            let mut all = Vec::new();
            for r in rows { all.push(r?); }
            fs::write(out_path, serde_json::to_string_pretty(&all)?)?;
            println!("📤 Exported to {}", out_path);
        }
>>>>>>> cd25f24 (feat: v0.2.0 - standalone feature set)
        _ => println!("Unknown command."),
    }

    Ok(())
}
