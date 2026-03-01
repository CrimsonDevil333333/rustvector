# RustVector 🦀⚡ v0.7.0

**RustVector** is a high-performance, standalone vector database engine built by **Satyaa & Clawdy**. Designed for edge computing on the **Raspberry Pi 5**, it provides a persistent "shared brain" for AI agents with near-zero overhead.

## 🚀 Key Features
- **🧠 Multi-Provider**: Support for **Ollama** (Local), **OpenAI**, and **Gemini**.
- **📊 Progress Tracking**: Real-time progress bars for ingestion.
- **📄 MarkItDown**: Auto-converts PDFs, Office docs, and more via `markitdown`.
- **🛠️ Management CLI**: New `ls` and `rm` commands to view and manage stored data.
- **🌍 System-Wide**: Install once via Cargo; share memory across all agent sessions.

## 📦 Installation

```bash
# 1. Install MarkItDown
pipx install markitdown

# 2. Build and Install RustVector
git clone https://github.com/CrimsonDevil333333/rustvector.git
cd rustvector
cargo install --path .
```

## 📖 Usage Guide

### 1. Configure Brain
```bash
rustvector config --provider ollama --model llama3.2:1b
```

### 2. Ingest Data (With Progress Bar!)
Index folders recursively. Non-text files auto-convert.
```bash
rustvector ingest /home/pi/workspace/docs
```

### 3. Manage Vectors
List stored data or delete specific entries:
```bash
# List top 10 entries
rustvector ls

# Delete entry by ID
rustvector rm 42
```

### 4. Semantic Search
```bash
rustvector search \"How do I configure the firewall?\"
```

---
*Built with 🦀 by Satyaa & Clawdy.*
