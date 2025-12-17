<p align="center">
  <img src="docs/img/mnemosyne.png" alt="Mnemosyne" height="120">
</p>

# Mnemosyne

A terminal-based proofreader that helps refine your texts through a quiet interface.

## What it aims to be

- A TUI tool for working with text documents, paragraph by paragraph
- An AI-assisted writing environment (left panel: conversation, right panel: document)
- A simple internal versioning system (no git dependency)

## What it actually is (for now)

A learning project. An excuse to finally give Rust a fair chance, or rather, to let Rust give me one.

The goal is to build something useful while actually getting the 'kung-fu' of ownership, borrowing, lifetimes, and all those things that tripped me up before. This time with a real project, not abstract exercises.

## Status

Most of the core functionality is implemented now. The interface works, operations respond as they should. Details could use polishing, but it's becoming something you can actually use.

[See it in action](docs/gallery/index.md)

## Installation

### Pre-compiled binaries

Pre-compiled binaries will be available in the next few days, after polishing some final details.

**Download the pre-compiled binary for your system:**

1. Go to the [Releases page](https://github.com/adrianpk/mnemosyne/releases)
2. Download the latest version for your operating system:
   - **Windows**: `mnemosyne-windows-x86_64.exe`
   - **macOS (Intel)**: `mnemosyne-macos-x86_64`
   - **macOS (Apple Silicon)**: `mnemosyne-macos-arm64`
   - **Linux**: `mnemosyne-linux-x86_64`
3. Move the file to a convenient location
4. On macOS/Linux, make it executable:
   ```bash
   chmod +x mnemosyne-macos-x86_64  # or the Linux version
   ```
5. Run it:
   - **Windows**: Double-click `mnemosyne-windows-x86_64.exe` or run from terminal
   - **macOS/Linux**: Open terminal and run `./mnemosyne-macos-x86_64 your-document.txt`

### For developers

If you have Rust installed, you can build from source:

```bash
git clone <repository-url>
cd mnemosyne
cargo build --release
./target/release/mnemosyne your-document.txt
```

## Configuration

### Setting your OpenAI API Key

**Recommended: Use the app's Settings (F11)**

Run the app and press `F11` to open Settings. Enter your API key and press `Ctrl+S` to save. The key will be stored securely at `~/.config/mnemosyne/config.toml` with permissions 600 (owner read/write only).

**Alternative methods:**

If you prefer to configure manually:

*Option 1: Edit config file directly*
```bash
mkdir -p ~/.config/mnemosyne
cp config.example.toml ~/.config/mnemosyne/config.toml
nano ~/.config/mnemosyne/config.toml  # Add your API key
```

*Option 2: Environment variable*
```bash
export OPENAI_MNEMOSYNE_API_KEY="sk-your-key"
```

Priority order: config file > `OPENAI_MNEMOSYNE_API_KEY` > `OPENAI_API_KEY`

## Usage

```bash
cargo run -- path/to/document.txt
```

For detailed documentation on all features and operations, see the [Usage Guide](docs/usage-guide.md).
