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

Early. Very early. Two panels on a screen and a lot of curiosity.

[See it in action](docs/gallery/index.md)

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
