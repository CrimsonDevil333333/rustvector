use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct VectorEntry {
    id: i32,
    content: String,
    metadata: String,
    embedding: Vec<u8>,
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * b_f32(y)).sum();
    dot_product // Assuming normalized vectors for simplicity in this MVP
}

// Helper to handle byte conversion
fn b_f32(val: &f32) -> f32 { *val }

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let db_path = "vector.db";
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
        println!("Usage: rustvector <command> [args]");
        println!("Commands: add <content> <metadata_json> <embedding_csv>, search <query_embedding_csv>");
        return Ok(());
    }

    match args[1].as_str() {
        "add" => {
            let content = &args[2];
            let metadata = &args[3];
            let embedding_str = &args[4];
            let embedding: Vec<f32> = embedding_str.split(',')
                .map(|s| s.parse().unwrap())
                .collect();
            let mut embedding_bytes = Vec::with_capacity(embedding.len() * 4);
            for f in embedding {
                embedding_bytes.extend_from_slice(&f.to_le_bytes());
            }
            conn.execute(
                "INSERT INTO vectors (content, metadata, embedding) VALUES (?1, ?2, ?3)",
                params![content, metadata, embedding_bytes],
            )?;
            println!("Added entry.");
        }
        "search" => {
            let query_str = &args[2];
            let query_vec: Vec<f32> = query_str.split(',')
                .map(|s| s.parse().unwrap())
                .collect();

            let mut stmt = conn.prepare("SELECT id, content, metadata, embedding FROM vectors")?;
            let rows = stmt.query_map([], |row| {
                let bytes: Vec<u8> = row.get(3)?;
                let embedding: Vec<f32> = bytes
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                    .collect();
                Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?, embedding))
            })?;

            let mut results: Vec<_> = Vec::new();
            for row in rows {
                let (content, metadata, embedding) = row?;
                let score = cosine_similarity(&query_vec, &embedding);
                results.push((content, metadata, score));
            }

            results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
            for (content, meta, score) in results.iter().take(5) {
                println!("[{:.4}] {}: {}", score, content, meta);
            }
        }
        _ => println!("Unknown command"),
    }

    Ok(())
}
