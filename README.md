# 󰏘 termux-theme (v0.1.2)

Fast, dynamic, and live-previewing color theme switcher for **Termux** written in **Rust**.

Developed by **Termux Tyro (@Remo773)**.

---

## ✨ Features

- **⚡ Instant Live Preview:** Preview themes instantly without restarting Termux using OSC escape sequences.
- **🔍 fzf Integration:** Interactive theme selector with real-time preview on navigation.
- **📱 Responsive Terminal Layout:** `list` and `--help` commands automatically adjust their layout to your screen width.
- **🛡️ Safe Backup & Restore:** Canceling (`q` or `Esc`) restores your previous theme, and saving creates a backup of `colors.properties`.
- **🚀 Ultra Fast:** Lightweight binary compiled with Rust for instant startup.

---

## 🚀 Quick Start & Installation

### Option 1: Using Pre-compiled Binaries
Copy the binary for your architecture from the `builds/` directory to your local bin path:

```bash
# For aarch64 (64-bit ARM mobile devices)
cp builds/termux-theme-rust-aarch64 ~/.local/bin/termux-theme
chmod +x ~/.local/bin/termux-theme
```

### Option 2: Build from Source
Ensure you have Rust installed on Termux (`pkg install rust`), then run:

```bash
git clone https://github.com/Remo773/termux-theme-rs.git
cd termux-theme-rs
cargo build --release
cp target/release/termux-theme ~/.local/bin/
```

---

## 📖 Usage & Commands

```text
USAGE:
  termux-theme [COMMAND | THEME_NAME | INDEX]

COMMANDS & OPTIONS:
  (no arguments)          Launch interactive fzf menu with live preview
  <name | number>         Preview & apply theme by name or index (e.g., termux-theme 5)
  list                    List available themes in responsive table format
  current                 Display current active theme
  random                  Pick & permanently save a random theme
  save-as <name>          Save current colors as a new theme file
  -h, --help              Display man-style help message
  -v, --version           Display version information
```

### 💡 Examples

```bash
# Open interactive fzf theme chooser with live preview
termux-theme

# Apply theme by index number
termux-theme 5

# Apply theme by name
termux-theme ubuntu

# Save current terminal colors as a custom theme
termux-theme save-as my-theme-name
```

---

## 🛠️ Requirements

- **Termux** (Android)
- **`fzf`** (for interactive menu): Install using `pkg install fzf`
- **`gum`** *(optional)*: For enhanced UI boxes (`pkg install gum`)

---

## 🌐 Cross Compilation

This project includes GitHub Actions for automated cross-compilation targeting:
- `aarch64-linux-android` (ARM 64-bit)
- `armv7-linux-androideabi` (ARM 32-bit)
- `x86_64-linux-android` (x86 64-bit)
- `i686-linux-android` (x86 32-bit)

---

## 👤 Author

- **Termux Tyro (@Remo773)**
