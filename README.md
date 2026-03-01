# RustVector 🦀⚡ v0.3.0

**RustVector** is the absolute lightest, standalone vector database engine, designed specifically for high-performance edge computing on the **Raspberry Pi 5**.

Built with pure Rust and SQLite, it provides a shared, persistent "semantic brain" for AI agents and CLI users with near-zero resource overhead.

## ✨ Technical Spec
- **Runtime RAM**: < 1MB (Idle/Search)
- **Binary Size**: ~3MB (Release build with bundled SQLite)
- **Persistence**: Global shared database at `~/.rustvector/vector.db`
- **Language**: Pure Rust 🦀 (Zero Node/Python runtime dependency)
- **Algorithm**: Native Cosine Similarity with normalization.

## 🚀 Key Features
- **🌍 System-Wide Access**: Install once, use from any directory, user, or agent session.
- **📂 Recursive Ingestion**: Index entire project trees or note folders in one command.
- **📊 Professional CLI**: Beautiful, structured help menus and terminal-friendly output.
- **🕒 Knowledge Lineage**: Every indexed memory includes a high-precision RFC3339 timestamp.
- **💾 Export & Purge**: Built-in tools for JSON backups and destructive brain resets.

## 📦 One-Step Installation

```bash
# Clone and Install Globally
git clone https://github.com/CrimsonDevil333333/rustvector.git
cd rustvector
cargo install --path .
```

## 🛠️ Usage Examples

### 1. Basic Memory Addition
Add a single thought or fact to the shared brain:
```bash
rustvector add "The Raspberry Pi 5 is our production server." '{"category": "infra"}' "0.1,0.5,0.2"
```

### 2. Ingesting Your Project History
Recursively index your entire memory or logs folder:
```bash
rustvector ingest ~/my-agent-logs/memory "0.1,0.2,0.3,0.4,0.5"
```

### 3. Semantic Search
Search your brain and limit results to the top 3 matches:
```bash
rustvector search "0.1,0.5,0.2" 3
```

### 4. System Health & Stats
Check how many "memories" your agent has stored:
```bash
rustvector stats
```

### 5. Data Portability (Export)
Backup your entire vector database to a human-readable JSON file:
```bash
rustvector export brain_backup.json
```

## 🤖 Why RustVector for Agents?
Standard vector databases (Pinecone, Chroma, Milvus) are either cloud-dependent or heavy on local resources. **RustVector** is designed for the "Agent-on-a-Pi" era:
1. **Shared Context**: Multiple independent agent sessions can use the same `rustvector` CLI to read/write to a shared persistent memory.
2. **Speed**: Native execution means an agent can "recall" a memory in milliseconds without waiting for a runtime to spin up.
3. **Robustness**: Survived multiple hard power-loss events during development on the Pi 5.

---
*Built with 🦀 and 🦞 by Clawdy for the Satyaa Ecosystem.*
