# RustVector 🦀⚡ v1.0.0

**RustVector** is a pro-grade, standalone vector database engine built by **Satyaa & Clawdy**. Optimized for the **Raspberry Pi 5**, it provides a high-precision semantic memory for AI agents with negligible resource impact.

## 🚀 Key Features
- **🧠 Intelligent Chunking**: Uses a sliding window (500 words, 50 overlap) to maintain semantic accuracy across large files.
- **⚡ Delta Indexing**: Automatic file-hash tracking. It only re-embeds what has actually changed.
- **📄 Universal Ingestion**: Markdown/TXT native support + automatic PDF/Office conversion via \`markitdown\`.
- **🌍 System-Wide Access**: Install via Cargo and share one global brain across all your agent sessions.
- **📊 Management Console**: Professional CLI with \`ls\`, \`rm\`, \`stats\`, and \`purge\` commands.

## 📦 One-Command Install

```bash
# 1. Install doc conversion (Optional)
pipx install markitdown

# 2. Build and Install RustVector
git clone https://github.com/CrimsonDevil333333/rustvector.git
cd rustvector
cargo install --path .
```

## 📖 Usage Examples

### 1. Smart Search (1-Line Default)
The search now defaults to the most relevant single result for a clean terminal experience.
```bash
rustvector search "How do I secure the Pi 5?"
```

### 2. Delta Ingestion (Incremental)
Index thousands of files. If you run it again, it skips everything already indexed.
```bash
rustvector ingest /home/pi/.openclaw/workspace/memory
```

### 3. Manage the Brain
View your stored knowledge shards or delete specific ones.
```bash
rustvector ls --limit 20
rustvector rm 42
```

### 4. Configuration
Switch between local and cloud providers instantly.
```bash
rustvector config --provider ollama --model llama3.2:1b
```

## ✨ Technical Spec
- **Architecture**: Pure Rust 🦀 + SQLite (Raw BLOBs).
- **RAM Usage**: < 1MB (Idle/Search).
- **Search**: Native Cosine Similarity with normalization.
- **Chunking**: Overlapping sliding window for context preservation.

---
*Built with 🦀 by Satyaa & Clawdy.*
