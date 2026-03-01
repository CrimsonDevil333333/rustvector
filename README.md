# RustVector 🦀⚡ v0.5.0

**RustVector** is a high-performance, ultra-lightweight standalone vector database engine built for the **Raspberry Pi 5**. It provides a persistent "shared brain" for AI agents and CLI power-users with near-zero overhead.

## 🚀 Key Features
- **🧠 Multi-Provider Support**: Switch between **Ollama** (Local), **OpenAI**, and **Google Gemini** seamlessly.
- **🌍 System-Wide Access**: Install once via Cargo and use from any directory or agent session.
- **📂 Recursive Ingestion**: Index entire project trees or documentation folders in a single command.
- **💾 Global Persistence**: All agents share the same database at `~/.rustvector/vector.db`.
- **🛠️ Zero Runtime Bloat**: Pure Rust + SQLite. Runs in **< 1MB RAM**.

## 📦 Installation (The Cargo Way)

To install **RustVector** globally on your system:

```bash
git clone https://github.com/CrimsonDevil333333/rustvector.git
cd rustvector
cargo install --path .
```

## 🛠️ Configuration

Configure your preferred embedding provider. Settings are persisted in `~/.rustvector/config.json`.

```bash
# Use Local Ollama (Default)
rustvector config --provider ollama --model all-minilm

# Use OpenAI
rustvector config --provider openai --model text-embedding-3-small --key YOUR_API_KEY

# Use Google Gemini
rustvector config --provider gemini --model embedding-001 --key YOUR_API_KEY
```

## 📖 Usage Examples

### 1. Store a Thought
```bash
rustvector add \"The Pi 5 is my production workhorse.\" '{\"category\": \"infra\"}'
```

### 2. Ingest a Project
```bash
rustvector ingest /home/pi/my-cool-project
```

### 3. Semantic Search
```bash
rustvector search \"What hardware am I using?\" --limit 3
```

### 4. Check Brain Stats
```bash
rustvector stats
```

## ✨ Why RustVector?
Most vector DBs are either heavy (Docker-required) or cloud-only. **RustVector** is built for the **Agentic Era**:
1. **Agent Interop**: If Agent A learns something, Agent B knows it instantly via the shared CLI.
2. **Pi-First**: Optimized for ARM64 and low-memory environments.
3. **Stand-alone**: No daemon, no server, no background processes. Just a binary.

---
*Built with 🦀 by Satyaa & Clawdy.*
