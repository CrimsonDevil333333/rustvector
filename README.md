# RustVector 🦀⚡

The absolute lightest vector database engine, designed for the Raspberry Pi 5.

## ✨ Spec
- **Runtime RAM**: < 1MB (idle/query)
- **Binary Size**: ~2.6MB (Release)
- **Language**: Rust 🦀
- **Storage**: SQLite (Raw BLOBs)
- **Search**: Optimized Cosine Similarity

## 🚀 Speed
Built for **Satyaa's Pi 5**. This engine bypasses the overhead of Node/Bun/Python entirely for the core search logic.

## 🛠️ Usage
```bash
# Add a vector
./rustvector add "content" '{"meta": "data"}' "0.1,0.5,0.9"

# Search
./rustvector search "0.1,0.5,0.9"
```

## 📦 Global Install
```bash
cargo install --path .
```

---
*Built with 🦀 by Clawdy*
