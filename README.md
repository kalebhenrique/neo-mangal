# neo-mangal ⚡

> Modern, blazing-fast Manga TUI Downloader and Kindle Comic Converter (KCC) processor built in Rust with `ratatui` and `crossterm`.

`neo-mangal` is an **aggressive refactor and modern reimagining** of the original [mangal](https://github.com/metafates/mangal) by **metafates**, rewritten from the ground up in Rust. It pairs with [**neo-mangal-scrapers**](https://github.com/kalebhenrique/neo-mangal-scrapers)—an external Lua scraper repository—to actively maintain, revitalize, and advance the terminal manga reading and Kindle conversion experience with zero friction.

---

## ✨ Features

- 🌐 **Modular Lua Scrapers ([neo-mangal-scrapers](https://github.com/kalebhenrique/neo-mangal-scrapers))**: Scrapers are decoupled from the Rust core into external Lua 5.4 scripts—providing instant updates without recompiling, and concurrent multi-source search.
- 📱 **Native Kindle Optimization**: Direct conversion to Kindle KF8 (`.azw3`) at 300 PPI via Kindle Comic Converter (KCC)
- 📖 **Unified Download Pipeline**: Flexible output targeting **AZW3**, **CBZ**, **EPUB**, and **MOBI**, with optional volume fusion to merge multi-chapter releases into clean single-volume archives.
- 🖼️ **Pure-Rust Terminal Cover Preview**: High-definition cover thumbnails rendered directly in the terminal using TrueColor halfblocks with dynamic window-aware scaling—zero external C dependencies (`chafa`-free).
- ⚡ **Modern TUI Architecture**: Built with `ratatui` and Tokio async tasks, featuring real-time progress bars, inline chapter filtering, wrap-around pagination, and clean immediate exit.

---

## 🌐 Manga Sources & Lua Scrapers Engine

`neo-mangal` delegates all web scraping logic to [**kalebhenrique/neo-mangal-scrapers**](https://github.com/kalebhenrique/neo-mangal-scrapers), powered by an embedded Lua 5.4 runtime (`mlua`).

```
neo-mangal (Rust Core)
    ├── Search & Download Manager
    ├── KCC & KindleGen Toolchain
    └── Embedded Lua 5.4 Runtime
           │
           ▼ loads from ~/.config/neo-mangal/sources/
    neo-mangal-scrapers (.lua files)
           ├── WeebCentral.lua
           ├── MangaDex.lua
           └── (Community Scrapers)
```

### Managing Sources via CLI

```bash
# Install or update scrapers from GitHub
neo-mangal sources install

# Reset scrapers and sync with local workspace
neo-mangal sources reset

# List all currently installed scrapers
neo-mangal sources list

# Print directory path of installed scrapers
neo-mangal sources path
```

On first launch, if no sources are found, `neo-mangal` will automatically open an installation dialog to download official scrapers with one click.

---

## 🔧 Prerequisites: KCC & Nerd Fonts

### 1. KCC CLI

For automated background conversions without touching the GUI:

```bash
# Install pipx (if not already installed)
brew install pipx

# Install KCC CLI in an isolated user environment
pipx install git+https://github.com/ciromattia/kcc.git
```

### 2. KindleGen via Amazon Kindle Previewer

Install **[Amazon Kindle Previewer](https://www.amazon.com/Kindle-Previewer/b?ie=UTF8&node=21381691011)** directly from Amazon:

- **Download**: [Amazon Kindle Previewer 3](https://www.amazon.com/Kindle-Previewer/b?ie=UTF8&node=21381691011)

> [!NOTE]
> `neo-mangal` automatically discovers `kindlegen` inside `/Applications/Kindle Previewer 3.app/Contents/lib/fc/bin/kindlegen`. No manual PATH configuration or file copying is required.

### 3. Nerd Fonts

`neo-mangal` uses [**Nerd Fonts**](https://www.nerdfonts.com/) glyphs for checkboxes, badges, and headers:

```bash
# Recommended: FiraCode or JetBrainsMono Nerd Font
brew install --cask font-fira-code-nerd-font
# or
brew install --cask font-jetbrains-mono-nerd-font
```

### Linux & Windows

- **Linux**: Download from [nerdfonts.com/font-downloads](https://www.nerdfonts.com/font-downloads) and copy to `~/.local/share/fonts/`.
- **Windows**: Install via `winget install JanDeDobbeleer.OhMyPosh` or download `.ttf` directly from Nerd Fonts.
- After installing, open your terminal preferences and select the installed font (e.g., `FiraCode Nerd Font`).

---

## 🚀 Installation & Build

```bash
# Clone the repository
git clone git@github.com:kalebhenrique/neo-mangal.git
cd neo-mangal

# Run test suite (20 unit & integration tests)
cargo test

# Launch neo-mangal
cargo run
```

Or build an optimized release binary:

```bash
cargo build --release
./target/release/neo-mangal
```

---

## 📄 License & Attribution

`neo-mangal` is licensed under the **MIT License**.  
Special credit to **metafates** and the contributors of [mangal](https://github.com/metafates/mangal) for pioneering terminal manga scraping.
