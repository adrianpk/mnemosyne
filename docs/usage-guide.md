# Mnemosyne - Usage Guide

## Overview

Mnemosyne is a terminal-based (TUI) writing analysis tool that helps improve text through AI-powered editorial operations.

---

## Getting Started

### Launch

**Current (Development):**
```bash
cargo run path/to/your-document.txt
```

**Future (Production):**
```bash
mnemosyne path/to/your-document.txt
```

### Requirements
- Rust toolchain (for development)
- OpenAI API key (configured via Settings or manually)
- Document must use double-newline paragraph separation (`\n\n`)

### Configuring Your API Key

**Recommended: Use Settings (F11)**

Press `F11` to open Settings, enter your OpenAI API key, and press `Ctrl+S` to save. The app will store it securely at `~/.config/mnemosyne/config.toml` with permissions 600 (owner read/write only).

**Alternative Configuration Methods:**

If you prefer to configure manually (though not necessary):

*Manual config file editing:*
```bash
mkdir -p ~/.config/mnemosyne
cp config.example.toml ~/.config/mnemosyne/config.toml
nano ~/.config/mnemosyne/config.toml  # Add your API key
```

*Environment variables:*
```bash
export OPENAI_MNEMOSYNE_API_KEY="sk-your-key"
# or
export OPENAI_API_KEY="sk-your-key"
```

**Priority order:** config file > `OPENAI_MNEMOSYNE_API_KEY` > `OPENAI_API_KEY`

**Security:**
- Config file is created with permissions `600` (owner read/write only)
- Prevents other users on the system from reading your API key
- As secure as environment variables, but less visible system-wide

---

## Navigation & Basic Controls

### Paragraph Selection
| Key            | Action                              |
| -------------- | ----------------------------------- |
| `↓` / `Ctrl+J` | Select next paragraph               |
| `↑` / `Ctrl+K` | Select previous paragraph           |
| `Ctrl+A`       | Toggle full document selection mode |

### Conversation Panel
| Key        | Action                   |
| ---------- | ------------------------ |
| `PageUp`   | Scroll conversation up   |
| `PageDown` | Scroll conversation down |

### Direct Edit Mode
| Key      | Action                                 |
| -------- | -------------------------------------- |
| `Ctrl+E` | Enter edit mode (sentence-by-sentence) |
| `Ctrl+S` | Apply edit                             |
| `Esc`    | Cancel edit                            |

### Version Control
| Key      | Action                                                      |
| -------- | ----------------------------------------------------------- |
| `Ctrl+Z` | Undo to previous version (saves to file)                    |
| `Ctrl+Y` | Redo to next version (saves to file)                        |
| `Ctrl+S` | Commit current state (saves and purges future versions)     |

### More Operations & Exit
| Key      | Action                          |
| -------- | ------------------------------- |
| `F11`    | Settings (not yet implemented)  |
| `F12`    | Open/Close More Operations menu |
| `Ctrl+Q` | Quit application                |

---

## Editorial Operations

Operations are triggered by:
- **Function keys (F2-F10)** for specific operations
- **Slash commands** (typed in left panel) - alternative way to trigger specific operations
- **Natural language input** (Freeform Editorial Mode) - for flexible, conversational interaction
- **F12 More menu** - reserved for future less-frequent operations

---

### Freeform Editorial Mode

**Default interaction mode when no command/function key is used**

Freeform Editorial Mode allows you to interact with Mnemosyne using natural language, without needing to specify commands or operations. The LLM infers your intent and responds accordingly.

**How it works:**
1. Type your request naturally in the input field (no slash command needed)
2. Press Enter
3. The LLM analyzes your intent and decides how to respond
4. The response type is determined automatically

**Response types (inferred by LLM):**
- **Replace:** Text that can be directly applied to the document (requires confirmation)
- **Suggest:** Multiple alternatives for you to choose from
- **Critique:** Analysis, guidance, or editorial feedback (no text changes)
- **None:** Pure conversational response (explanations, questions, reflections)

**Scope:**
By default, Freeform operates on the **selected paragraph**. To work on the full document, use any of these methods:

1. **`Ctrl+A`**: Toggle full document selection visually (all paragraphs highlighted, stays active)
2. **`/all` or `/full` prefix**: Type `/all your prompt here` or `/full your prompt here` (works once per command, any language)
3. **Mention in text**: Use "full document", "entire text", "documento completo", etc. in your prompt

Note: Only `Ctrl+A` provides visual feedback (highlighting). The `/all` prefix and text mentions work invisibly.

**Examples:**

*Editorial action (expects text change):*
```
> Make this paragraph sound more optimistic
→ LLM provides rewritten version in Replace mode
→ Shows diff for confirmation (Enter/Esc)
```

*Editorial guidance (expects advice):*
```
> How can I make this sound more optimistic?
→ LLM provides guidance in Critique mode
→ Shows suggestions but no text replacement
```

*Creative rewrite:*
```
> Rewrite this as if written by Virginia Woolf
→ LLM provides alternatives in Suggest mode
→ Shows numbered options to choose from
```

*Conversational:*
```
> What do you think about this paragraph?
→ LLM responds in None mode
→ Shows conversational feedback only
```

*Full document scope (Method 1: Ctrl+A):*
```
Press Ctrl+A (all paragraphs highlighted)
> Make the tone more consistent throughout
→ LLM works on entire document
Press Ctrl+A again to return to single paragraph mode
```

*Full document scope (Method 2: /all or /full prefix):*
```
> /all What is the overall tone of this text?
> /full Can you simplify the language?
→ LLM analyzes entire document
→ Works in any language, no need to press Ctrl+A
```

**Key principles:**
- The LLM never applies changes automatically; you always have control
- The system declares what type of response it's providing
- You decide whether to accept, select, or simply read the feedback
- Your writing voice and intent are always preserved

**When to use Freeform vs. Operations:**
- **Use Freeform when:** You want flexibility, creative rewrites, or conversational feedback
- **Use Operations (F2-F10) when:** You want predictable, standardized results (grammar correction, style polish, etc.)

---

### F1 - Help
**Scope:** Application-wide
**Behavior:** Toggle help screen

- Displays a quick reference cheatsheet with all keybindings and operations
- Press `F1` or `q` to close
- Available from any mode

**Sections:**
- Navigation keybindings
- Editorial operations (F2-F10)
- Freeform Editorial Mode usage
- Version control commands
- Direct editing commands

---

### F2 - Summary
**Command:** `/summary` (can also be typed)
**Scope:** Full document
**Behavior:** Informative only (no changes applied)

- Generates a concise summary of the entire document
- **Automatically saves** summary to: `<filename>-summary.txt`
- Useful for generating abstracts or checking document coherence
- Does not modify the original document

**Example:**
```
Press F2 → Wait for LLM
Input:  my-essay.txt
Output: my-essay-summary.txt (auto-saved)
        Summary displayed in conversation panel
```

---

### F3 - Critique
**Command:** `/critique`
**Scope:** Full document
**Behavior:** Informative only (no changes applied)

- Professional editorial critique without rewriting
- Identifies structural issues, pacing problems, weak arguments
- Provides actionable feedback for manual revision

**Expected output:**
- List of editorial observations
- No automatic text changes

---

### F4 - Repeats
**Command:** `/repeats`
**Scope:** Full document
**Behavior:** Browse mode (interactive)

- Detects repeated words/phrases across the **entire document** (global analysis)
- Shows frequency count for each repeated term
- Allows interactive highlighting by selecting a term

**Workflow:**
1. Press F4
2. See list: `[1] term1`, `[2] term2`, etc.
3. Type number to highlight that term throughout the document
4. Press `Esc` to exit browse mode

---

### F5 - Style
**Command:** `/style`
**Scope:** Selected paragraph
**Behavior:** Auto-apply if safe, otherwise Review mode

- Improves stylistic flow, rhythm, and readability
- Preserves author's voice and intent
- May show diff for user confirmation if changes are significant

**Workflow:**
1. Select paragraph with arrows
2. Press F5
3. Review suggestion (if prompted)
4. Press `Enter` to accept or `Esc` to reject

---

### F6 - Grammar
**Command:** `/grammar`
**Scope:** Selected paragraph
**Behavior:** Auto-apply if safe, otherwise Review mode

- Corrects grammar, spelling, punctuation, syntax
- Only fixes errors, does not restyle
- Conservative: only applies obvious corrections automatically

**Workflow:**
1. Select paragraph with arrows
2. Press F6
3. Review suggestion (if prompted)
4. Press `Enter` to accept or `Esc` to reject

---

### F7 - Rephrase
**Command:** `/rephrase`
**Scope:** Selected paragraph
**Behavior:** Select from alternatives

- Provides 2-3 alternative rewrites of the selected paragraph
- Focuses on clarity and flow
- User must explicitly choose one alternative

**Workflow:**
1. Select paragraph with arrows
2. Press F7
3. See alternatives: `[1] option1`, `[2] option2`, etc.
4. Type number to apply that alternative
5. Press `Esc` to cancel

---

### F8 - Thesaurus
**Command:** `/thesaurus`
**Scope:** Selected paragraph
**Behavior:** Informative only (no changes applied)

- Identifies weak, vague, or imprecise word choices
- Suggests context-appropriate lexical alternatives
- Format: `'word' → alt1, alt2 — explanation`

**Expected output:**
- List of word suggestions
- No automatic text changes (user applies manually)

---

### F9 - Overuse
**Command:** `/overuse`
**Scope:** Selected paragraph
**Behavior:** Informative only (no changes applied)

- Flags clichéd, editorially fatigued, or overused expressions
- Helps identify stale language patterns
- Does not provide replacements (use Thesaurus or Rephrase for that)

**Expected output:**
- List of overused terms/phrases
- No automatic text changes

---

### F10 - Echoes
**Command:** `/echoes`
**Scope:** Full document
**Behavior:** Browse mode (interactive)

- Detects repetitions in **close proximity** (same sentence, adjacent paragraphs)
- Different from Repeats: focuses on local redundancy, not global frequency
- Allows interactive highlighting by selecting a term

**Workflow:**
1. Press F10
2. See list: `[1] echo1`, `[2] echo2`, etc.
3. Type number to highlight that echo throughout the document
4. Press `Esc` to exit browse mode

**Difference from Repeats:**
- **Repeats (F4):** Global analysis - finds ALL repetitions across document
- **Echoes (F10):** Local analysis - finds repetitions close together

---

### F11 - Settings
**Scope:** Application configuration
**Behavior:** Edit settings interactively

**What you can configure:**
- OpenAI API Key (stored in `~/.config/mnemosyne/config.toml`)

**How to use:**
1. Press F11 to open Settings
2. Enter or edit your API key in the text field
3. Press `Ctrl+S` to save changes
4. Press `Esc` to cancel without saving

**Security:**
- Settings are saved to `~/.config/mnemosyne/config.toml`
- File permissions are automatically set to `600` (owner read/write only)
- Your API key is not exposed to other users on the system

**Note:** You can also edit the config file manually if preferred, but F11 Settings is the recommended way.

---

### F12 - More Operations
**Scope:** Additional operations menu
**Behavior:** Opens modal for future operations

Press `F12` to open the More Operations modal. Currently empty but available for future less-frequently used operations.

**Closing the modal:**
- Press `F12` again (toggle)
- Press `q`

---

## Versioning System

### How It Works

Every time you **accept a change** (from Grammar, Style, Rephrase, or Edit mode), Mnemosyne:

1. **Saves a snapshot** of the document BEFORE the change
2. **Applies the change** to the document
3. **Writes the updated document** to the original file
4. **Shows version info** (e.g., "Change accepted. v2/2")

### Version Storage

Versions are stored on disk in a hidden directory:

```
your-document.txt                      ← Current version (always up-to-date)
.mnemosyne/
  └── versions/
      └── your-document.txt/
          ├── 000-20251216-143022.txt  ← Snapshot before 1st change
          ├── 001-20251216-143145.txt  ← Snapshot before 2nd change
          └── ...
```

**Format:** `<version>-<timestamp>.txt`

### Undo/Redo

- **Ctrl+Z:** Undo to previous version (loads from disk AND saves to file)
- **Ctrl+Y:** Redo to next version (loads from disk AND saves to file)
- **Ctrl+S:** Commit current state (saves to file AND purges future versions)

**Behavior:**
- Undo/Redo automatically save the loaded version to your file
- When you exit and re-enter, you see exactly the version you were on
- The file always reflects what you see on screen

**Committing to a past version:**
If you undo from v7 to v3 and press **Ctrl+S** without making changes:
- Saves v3 to file (no new version created)
- Purges v4, v5, v6, v7 from disk permanently
- You're now at v3/3 (not v3/7)

**Linear History:**
- If you undo and then make a new change, all future versions are discarded automatically
- If you undo and press Ctrl+S, you explicitly discard future versions without making changes
- No branching or complex undo trees (intentionally simple)

### Version Info

Displayed in conversation panel after each change:
- `v1/1` - Original document, no changes yet
- `v2/2` - After first accepted change
- `v3/3` - After second accepted change
- etc.

### Replay Capability

Because versions are saved as complete files on disk, you can:
- Replay document evolution by reading files in order
- Compare any two versions manually using `diff`
- Restore to any historical state by copying a version file

---

## File Saving Behavior

### Automatic Saves

| Trigger                            | What Gets Saved                     | Where                                  |
| ---------------------------------- | ----------------------------------- | -------------------------------------- |
| Accept Grammar/Style suggestion    | Version snapshot + updated document | `.mnemosyne/versions/` + original file |
| Apply Rephrase alternative         | Version snapshot + updated document | `.mnemosyne/versions/` + original file |
| Accept manual edit (Ctrl+E)        | Version snapshot + updated document | `.mnemosyne/versions/` + original file |
| Run Summary operation (F2)         | Summary text                        | `<filename>-summary.txt`               |

### Auto-Save by Default

All changes are saved **automatically** when accepted. No manual save operation is needed.

---

## Typical Workflows

### Workflow 1: Grammar & Style Pass

```
1. Load document:  cargo run my-essay.txt
2. Select first paragraph (arrow keys)
3. Press F6 (Grammar)
4. Review suggestion → Enter to accept
5. Press F5 (Style)
6. Review suggestion → Enter to accept
7. Move to next paragraph (arrow down)
8. Repeat steps 3-7 for all paragraphs
```

**Result:** Document is corrected paragraph-by-paragraph with full version history.

---

### Workflow 2: Remove Repetitions

```
1. Load document:  cargo run my-essay.txt
2. Press F4 (Repeats) to see global repetitions
3. Type number to highlight a repeated term
4. Manually edit affected paragraphs (Ctrl+E)
5. Press F10 (Echoes) to check local redundancy
6. Fix close-proximity echoes as needed
```

**Result:** Document has fewer repetitions and better lexical variety.

---

### Workflow 3: Generate Summary

```
1. Load document:  cargo run my-essay.txt
2. Press F2
3. Wait for LLM response
4. Summary appears in conversation panel
5. Summary is automatically saved to: my-essay-summary.txt
```

**Alternative:** Type `/summary` and press Enter instead of F2.

**Result:** Summary file created without manual intervention.

---

### Workflow 4: Experiment with Undo/Redo

```
1. Load document:  cargo run my-essay.txt
2. Make several changes (Grammar, Style, etc.)
3. Press Ctrl+Z to undo last change
4. Press Ctrl+Z again to undo previous change
5. Press Ctrl+Y to redo
6. Make a new change (discards future versions)
```

**Result:** Linear history maintained, easy to explore different versions.

---

## Operation Modes

The app operates in different modes depending on the operation:

### Normal Mode
- Default state
- Navigate paragraphs with arrow keys
- Type commands or trigger operations with F-keys

### Review Mode
- Triggered by Grammar, Style operations (when change needs confirmation)
- Shows diff: original text vs. proposed text
- `Enter` to accept, `Esc` to reject

### Select Alternative Mode
- Triggered by Rephrase operation
- Shows numbered alternatives `[1]`, `[2]`, etc.
- Type number to apply, `Esc` to cancel

### Edit Mode
- Triggered by `Ctrl+E`
- Direct text editing with multi-line textarea
- Text is split into sentences (one per line) for easier editing
- `Ctrl+S` to apply, `Esc` to cancel

### Browse Repeats Mode
- Triggered by F4 (Repeats) or F10 (Echoes)
- Shows numbered list of repeated terms
- Type number to highlight term throughout document
- `Esc` to exit

---

## Tips & Best Practices

### Paragraph Scope vs. Full Document
- **Paragraph operations** (F5-F9): Work on selected paragraph only
- **Full document operations** (F2-F4, F10): Analyze entire document at once

When a full-doc operation is active, all paragraphs are visually highlighted.

### Accept/Reject Decisions
- **Grammar/Style:** Accept if the change preserves your voice
- **Rephrase:** Only use if you genuinely prefer an alternative
- **Manual Edit:** Use for precise control when AI suggestions miss the mark

### When to Use Each Operation
- **Help (F1):** Anytime you need a quick reference of keybindings and operations
- **Freeform Mode:** When you want flexible interaction, creative rewrites, or conversational feedback
- **Summary (F2):** To generate abstracts or check document coherence
- **Grammar (F6):** First pass - fix obvious errors
- **Style (F5):** Second pass - improve flow and readability
- **Rephrase (F7):** When a paragraph feels awkward but you're not sure how to fix it
- **Thesaurus (F8):** When you notice weak word choices
- **Overuse (F9):** When you suspect clichés or tired expressions
- **Repeats (F4):** After full draft is done, to catch global repetitions
- **Echoes (F10):** During revision, to catch local redundancy
- **Critique (F3):** Before final polish, to get structural feedback
- **Settings (F11):** To configure your OpenAI API key securely

---
