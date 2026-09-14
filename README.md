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

### Option 1: Download Pre-compiled Binary (Releases)
Download the latest binary for your architecture from [GitHub Releases](https://github.com/remo7777/termux-theme-rs/releases/latest):

```bash
# Example for aarch64 (64-bit ARM mobile devices)
curl -LO https://github.com/remo7777/termux-theme-rs/releases/latest/download/termux-theme-rust-aarch64
chmod +x termux-theme-rust-aarch64
mv termux-theme-rust-aarch64 ~/.local/bin/termux-theme
```

### Option 2: Build from Source on Termux

#### Prerequisites
Install Rust, C compiler (`clang`), `fzf`, and `gum` on Termux:
```bash
pkg update && pkg install rust clang fzf gum -y
```

#### Build & Install
1. Clone the repository:
   ```bash
   git clone https://github.com/remo7777/termux-theme-rs.git
   cd termux-theme-rs
   ```

2. Build optimized release binary (no extra `.cargo` config needed):
   ```bash
   cargo build --release
   ```

3. Copy the compiled binary to your PATH:
   ```bash
   mkdir -p ~/.local/bin
   cp target/release/termux-theme ~/.local/bin/
   chmod +x ~/.local/bin/termux-theme
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
