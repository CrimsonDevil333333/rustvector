# RustVector 🦀⚡ v0.4.0

**RustVector** is a high-performance, standalone vector database engine built for edge devices like the **Raspberry Pi 5**. It provides a shared, persistent "semantic brain" for AI agents with near-zero overhead.

## 🚀 Key Features
- **🧠 Multi-Provider Support**: Choose between local (**Ollama**) or cloud (**OpenAI**) embedding engines.
- **🌍 System-Wide Access**: Install once, use from any directory or agent session.
- **📂 Recursive Ingestion**: Index entire project trees in one command.
- **📊 Professional CLI**: Beautiful, structured help menus via `clap`.
- **🛠️ Configurable**: Persist your provider, model, and API keys locally.

## 📦 Installation
```bash
git clone https://github.com/CrimsonDevil333333/rustvector.git
cd rustvector
cargo install --path .
```

## 🛠️ Usage & Config

### 1. Configure your Provider
By default, it uses local Ollama. You can switch to OpenAI or change models easily:
```bash
# Switch to OpenAI
rustvector config --provider openai --model text-embedding-3-small --key your_api_key

# Switch back to local Ollama
rustvector config --provider ollama --model all-minilm
```

### 2. Store Memories (Natural Language)
No manual vector passing required. It handles the embedding automatically.
```bash
rustvector add "The Pi 5 is a production-grade server." '{"type": "infra"}'
```

### 3. Semantic Search
```bash
rustvector search "What hardware are we using?" --limit 3
```

### 4. Folder Ingestion
```bash
rustvector ingest /path/to/your/notes
```

### 5. Stats & Health
```bash
rustvector stats
```

## ✨ Technical Spec
- **RAM Usage**: < 1MB
- **Binary Size**: ~3MB
- **Database**: SQLite (Raw BLOBs)
- **Persistence**: `~/.rustvector/vector.db`

---
*Built with 🦀 and 🦞 by Clawdy for the Satyaa Ecosystem.*
