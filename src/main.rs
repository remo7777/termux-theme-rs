use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn get_termux_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/data/data/com.termux/files/home".to_string());
    PathBuf::from(home).join(".termux")
}

fn get_themes_dir() -> PathBuf {
    get_termux_dir().join("themes")
}

fn get_color_file() -> PathBuf {
    get_termux_dir().join("colors.properties")
}

fn clean_theme_name(raw: &str) -> String {
    let mut s = raw.trim();
    if let Some(pos) = s.find(". ") {
        s = &s[pos + 2..];
    }
    if let Some(pos) = s.find(' ') {
        s = &s[..pos];
    }
    s.trim_end_matches(".properties").to_string()
}

fn compile_osc_from_content(content: &str) -> String {
    let mut osc = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ['=', ':']).collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let val = parts[1].trim();
            if key == "background" {
                osc.push_str(&format!("\x1b]11;{}\x1b\\", val));
            } else if key == "foreground" {
                osc.push_str(&format!("\x1b]10;{}\x1b\\", val));
            } else if key == "cursor" {
                osc.push_str(&format!("\x1b]12;{}\x1b\\", val));
            } else if let Some(num_part) = key.strip_prefix("color")
                && num_part.chars().all(|c| c.is_ascii_digit())
            {
                osc.push_str(&format!("\x1b]4;{};{}\x1b\\", num_part, val));
            }
        }
    }
    osc
}

fn apply_osc_fast(theme_input: &str) -> bool {
    let theme_name = clean_theme_name(theme_input);
    let themes_dir = get_themes_dir();
    let prop_file = themes_dir.join(format!("{}.properties", theme_name));
    let osc_file = themes_dir.join(format!("{}.osc", theme_name));

    if !prop_file.exists() {
        return false;
    }

    let need_compile = match (prop_file.metadata(), osc_file.metadata()) {
        (Ok(pm), Ok(om)) => {
            if let (Ok(pt), Ok(ot)) = (pm.modified(), om.modified()) {
                pt > ot
            } else {
                true
            }
        }
        _ => true,
    };

    if need_compile
        && let Ok(content) = fs::read_to_string(&prop_file)
    {
        let osc_data = compile_osc_from_content(&content);
        let _ = fs::write(&osc_file, osc_data);
    }

    if let Ok(osc_data) = fs::read_to_string(&osc_file) {
        if let Ok(mut tty) = fs::OpenOptions::new().write(true).open("/dev/tty") {
            let _ = tty.write_all(osc_data.as_bytes());
            let _ = tty.flush();
        } else {
            let _ = io::stdout().write_all(osc_data.as_bytes());
            let _ = io::stdout().flush();
        }
        true
    } else {
        false
    }
}

fn get_available_themes() -> Vec<String> {
    let themes_dir = get_themes_dir();
    let mut themes = Vec::new();
    if let Ok(entries) = fs::read_dir(themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("properties")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                themes.push(stem.to_string());
            }
        }
    }
    themes.sort();
    themes
}

fn get_current_active_theme(themes: &[String]) -> Option<String> {
    let color_file = get_color_file();
    if !color_file.exists() {
        return None;
    }
    let current_content = fs::read(&color_file).ok()?;
    let themes_dir = get_themes_dir();
    for t in themes {
        let prop_file = themes_dir.join(format!("{}.properties", t));
        if let Ok(theme_content) = fs::read(&prop_file)
            && theme_content == current_content
        {
            return Some(t.clone());
        }
    }
    None
}

fn restore_original(original_content: Option<&str>) {
    if let Some(content) = original_content {
        let osc_data = compile_osc_from_content(content);
        if let Ok(mut tty) = fs::OpenOptions::new().write(true).open("/dev/tty") {
            let _ = tty.write_all(osc_data.as_bytes());
            let _ = tty.flush();
        } else {
            let _ = io::stdout().write_all(osc_data.as_bytes());
            let _ = io::stdout().flush();
        }
    }
}

fn save_permanently(raw_name: &str) {
    let theme_name = clean_theme_name(raw_name);
    let themes_dir = get_themes_dir();
    let prop_file = themes_dir.join(format!("{}.properties", theme_name));
    let color_file = get_color_file();

    if color_file.exists() {
        let bak = color_file.with_extension("properties.bak");
        let _ = fs::copy(&color_file, bak);
    }

    if prop_file.exists() {
        if fs::copy(&prop_file, &color_file).is_ok() {
            println!("\n\x1b[32m[✔]\x1b[0m Applied '\x1b[1m{}\x1b[0m' → ~/.termux/colors.properties", theme_name);
        } else {
            eprintln!("\n\x1b[31m[✘]\x1b[0m Error copying theme file.");
        }
    } else {
        eprintln!("\n\x1b[31m[✘]\x1b[0m Error: Theme file '{}.properties' not found.", theme_name);
    }
}

fn get_terminal_width() -> usize {
    if let Ok(cols) = env::var("COLUMNS")
        && let Ok(w) = cols.parse::<usize>()
    {
        return w;
    }

    if let Ok(tty) = fs::File::open("/dev/tty") {
        if let Ok(output) = Command::new("stty").arg("size").stdin(tty).output()
            && output.status.success()
        {
            let s = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = s.trim().split_whitespace().collect();
            if parts.len() == 2 && let Ok(cols) = parts[1].parse::<usize>() {
                return cols;
            }
        }
    }

    if let Ok(output) = Command::new("tput").arg("cols").output()
        && output.status.success()
    {
        let s = String::from_utf8_lossy(&output.stdout);
        if let Ok(w) = s.trim().parse::<usize>() {
            return w;
        }
    }

    80
}

fn print_help(width: usize) {
    let md = "\x1b[1;36m";
    let me = "\x1b[0m";
    let us = "\x1b[4;33m";
    let ue = "\x1b[24m";
    let cmd = "\x1b[1;32m";

    println!("{us} 󰏘 termux-theme v0.1.2 {ue}  Dynamic Termux Color Switcher");
    println!("\x1b[90mAuthor: Termux Tyro (@Remo773)\x1b[0m\n");

    println!("{md}NAME{me}");
    println!("    termux-theme - preview & apply Termux color palettes dynamically\n");

    println!("{md}SYNOPSIS{me}");
    println!("    {cmd}termux-theme{me} [{cmd}COMMAND{me} | {us}THEME_NAME{ue} | {us}INDEX{ue}]\n");

    println!("{md}COMMANDS & OPTIONS{me}");

    let options = [
        ("(no arguments)", "Launch interactive fzf menu with live preview"),
        ("<name | number>", "Preview & apply theme by name or index"),
        ("list", "List available themes in responsive table format"),
        ("current", "Display current active theme"),
        ("random", "Pick & permanently save a random theme"),
        ("save-as <name>", "Save current colors as a new theme file"),
        ("-h, --help", "Display this man-style help message"),
        ("-v, --version", "Display version information"),
    ];

    let w1 = 22;
    let gap = 2;
    let w2 = if width > w1 + gap + 4 {
        width - w1 - gap - 4
    } else {
        25
    };

    for (cmd_str, desc_str) in options {
        let mut desc_lines = Vec::new();
        let mut current_words = Vec::new();
        let mut current_len = 0;

        for word in desc_str.split_whitespace() {
            if current_len + word.len() + (if current_len > 0 { 1 } else { 0 }) <= w2 {
                current_words.push(word);
                current_len += word.len() + (if current_len > word.len() { 1 } else { 0 });
            } else {
                if !current_words.is_empty() {
                    desc_lines.push(current_words.join(" "));
                    current_words.clear();
                }
                current_words.push(word);
                current_len = word.len();
            }
        }
        if !current_words.is_empty() {
            desc_lines.push(current_words.join(" "));
        }
        if desc_lines.is_empty() {
            desc_lines.push(String::new());
        }

        for (idx, desc_line) in desc_lines.iter().enumerate() {
            let c1_colored = if idx == 0 {
                let pad = " ".repeat(w1.saturating_sub(cmd_str.len()));
                format!("{}{}{}{}", cmd, cmd_str, me, pad)
            } else {
                " ".repeat(w1)
            };

            println!("  {}  {}", c1_colored, desc_line);
        }
    }
    println!();

    println!("{md}EXAMPLES{me}");
    println!("  \x1b[38;5;75m󰅂\x1b[0m termux-theme 5");
    println!("  \x1b[38;5;75m󰅂\x1b[0m termux-theme ubuntu");
    println!("  \x1b[38;5;75m󰅂\x1b[0m termux-theme save-as dark-custom");
}

fn print_themes_grid(themes: &[String], active_theme: Option<&String>, width: usize) {
    println!("\x1b[1;33m󰏘\x1b[0m \x1b[1mAvailable themes in ~/.termux/themes/:\x1b[0m");

    if themes.is_empty() {
        return;
    }

    let max_name_len = themes.iter().map(|t| t.len()).max().unwrap_or(15);
    let item_cell_width = max_name_len + 8;

    let num_cols = if width >= item_cell_width * 2 { 2 } else { 1 };
    let col_width = width / num_cols;

    for chunk in themes.chunks(num_cols) {
        let mut line = String::new();
        for (col_idx, t) in chunk.iter().enumerate() {
            let global_idx = themes.iter().position(|x| x == t).unwrap_or(0) + 1;
            let is_active = Some(t) == active_theme;

            let formatted_item = if is_active {
                format!(" \x1b[38;5;75m󰅂\x1b[0m \x1b[1;32m{:>2}. {}\x1b[0m \x1b[32m\x1b[0m", global_idx, t)
            } else {
                format!(" \x1b[38;5;75m󰅂\x1b[0m {:>2}. {}", global_idx, t)
            };

            let visible_len = 6 + format!("{:>2}. {}", global_idx, t).len() + if is_active { 2 } else { 0 };
            let pad_len = if col_idx < chunk.len() - 1 && col_width > visible_len {
                col_width - visible_len
            } else {
                0
            };

            line.push_str(&formatted_item);
            line.push_str(&" ".repeat(pad_len));
        }
        println!("{}", line);
    }
}

fn main() {
    let _ = fs::create_dir_all(get_themes_dir());
    let args: Vec<String> = env::args().collect();
    let width = get_terminal_width();

    if args.len() >= 3 && args[1] == "preview" {
        apply_osc_fast(&args[2]);
        return;
    }

    let themes = get_available_themes();
    if themes.is_empty() {
        eprintln!("󰅙 No theme files found in {:?}", get_themes_dir());
        std::process::exit(1);
    }

    let active_theme = get_current_active_theme(&themes);
    let original_color_content = fs::read_to_string(get_color_file()).ok();

    if args.len() >= 2 {
        let sub = &args[1];
        match sub.as_str() {
            "-h" | "--help" | "help" => {
                print_help(width);
                return;
            }
            "-v" | "--version" => {
                println!("termux-theme v0.1.2");
                return;
            }
            "current" => {
                if let Some(act) = active_theme {
                    println!("\x1b[1;33m󰏘\x1b[0m Active theme: \x1b[1;32m{}\x1b[0m \x1b[32m\x1b[0m", act);
                } else {
                    println!("\x1b[1;33m󰏘\x1b[0m Active colors file does not match any theme in ~/.termux/themes/");
                }
                return;
            }
            "save-as" => {
                if args.len() < 3 {
                    eprintln!("Usage: termux-theme save-as <theme_name>");
                    std::process::exit(1);
                }
                let new_theme = args[2].trim_end_matches(".properties");
                let target_path = get_themes_dir().join(format!("{}.properties", new_theme));
                let color_file = get_color_file();
                if color_file.exists() {
                    if fs::copy(&color_file, &target_path).is_ok() {
                        println!("\x1b[32m[✔]\x1b[0m Saved current colors as '{}' in {:?}", new_theme, target_path);
                    } else {
                        eprintln!("\x1b[31m[✘]\x1b[0m Failed to save theme.");
                    }
                } else {
                    eprintln!("\x1b[31m[✘]\x1b[0m Error: ~/.termux/colors.properties not found.");
                }
                return;
            }
            "random" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
                let idx = (nanos as usize) % themes.len();
                let selected = &themes[idx];
                apply_osc_fast(selected);
                save_permanently(selected);
                return;
            }
            "list" => {
                print_themes_grid(&themes, active_theme.as_ref(), width);
                return;
            }
            _ => {
                let mut clean_arg = clean_theme_name(sub);
                if let Ok(num) = clean_arg.parse::<usize>()
                    && num >= 1
                    && num <= themes.len()
                {
                    clean_arg = themes[num - 1].clone();
                }

                if !themes.contains(&clean_arg) {
                    eprintln!("\x1b[31m󰅙\x1b[0m Unknown theme '{}'.", sub);
                    eprintln!("Available themes: {}", themes.join(", "));
                    std::process::exit(1);
                }

                apply_osc_fast(&clean_arg);

                let gum_check = Command::new("which").arg("gum").stdout(Stdio::null()).status();
                if gum_check.map(|s| s.success()).unwrap_or(false) {
                    let pill = Command::new("gum")
                        .args(["style", "--foreground", "16", "--background", "51", "--bold", "--padding", "0 1", " 󰍉 PREVIEW MODE "])
                        .output()
                        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                        .unwrap_or_default();
                    let box_styled = Command::new("gum")
                        .args([
                            "style",
                            "--border", "rounded",
                            "--border-foreground", "51",
                            "--padding", "0 2",
                            &format!("Theme: \x1b[1;38;5;220m{}\x1b[0m", clean_arg),
                            "",
                            "\x1b[1;38;5;46m󰄬 [Enter]\x1b[0m Save     \x1b[1;38;5;196m󰅙 [q/Esc]\x1b[0m Cancel",
                        ])
                        .output()
                        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                        .unwrap_or_default();
                    let _ = Command::new("gum")
                        .args(["join", "--vertical", &pill, &box_styled])
                        .status();
                } else {
                    println!("\n\x1b[1;30;46m 󰍉 PREVIEW MODE \x1b[0m");
                    println!("Theme: \x1b[1;33m{}\x1b[0m", clean_arg);
                    println!(" \x1b[1;32m󰄬 [Enter]\x1b[0m Save     \x1b[1;31m󰅙 [q/Esc]\x1b[0m Cancel");
                }

                let mut key = [0u8; 1];
                let _ = io::stdin().read_exact(&mut key);
                if key[0] == b'q' || key[0] == b'Q' || key[0] == 0x1b {
                    restore_original(original_color_content.as_deref());
                    println!("\n\x1b[31m󰅙\x1b[0m Cancelled. Restored original colors.");
                } else {
                    save_permanently(&clean_arg);
                }
                return;
            }
        }
    }

    let fzf_check = Command::new("which").arg("fzf").stdout(Stdio::null()).status();
    if !fzf_check.map(|s| s.success()).unwrap_or(false) {
        eprintln!("\x1b[31m[✘]\x1b[0m Error: 'fzf' is required but not installed.");
        eprintln!("Please install it using: \x1b[1;32mpkg install fzf\x1b[0m");
        std::process::exit(1);
    }

    let exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("termux-theme"));
    let exe_str = exe.to_string_lossy();

    let mut options_input = String::new();
    for (i, t) in themes.iter().enumerate() {
        let num = i + 1;
        if Some(t) == active_theme.as_ref() {
            options_input.push_str(&format!("{}. {} \x1b[32m\x1b[0m\n", num, t));
        } else {
            options_input.push_str(&format!("{}. {}\n", num, t));
        }
    }

    let child = Command::new("fzf")
        .env("FZF_DEFAULT_OPTS", "--height=40% --color fg:#ffffff,hl:#98c379,fg+:#ffffff,bg+:#222222,hl+:#98c379,prompt:#61afef,pointer:#e5c07b,marker:#56b6c2 --ansi --reverse --no-info --no-scrollbar --pointer=' '")
        .arg("--prompt=Select Termux Colors ➤ ")
        .arg("--exit-0")
        .arg("--border")
        .arg("--margin=1,5%")
        .arg(format!("--bind=focus:execute-silent('{}' preview {{}})", exe_str))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn();

    if let Ok(mut fzf_proc) = child {
        if let Some(mut stdin) = fzf_proc.stdin.take() {
            let _ = stdin.write_all(options_input.as_bytes());
        }
        if let Ok(out) = fzf_proc.wait_with_output()
            && out.status.success()
        {
            let selected = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !selected.is_empty() {
                save_permanently(&selected);
                return;
            }
        }
    }
    restore_original(original_color_content.as_deref());
    println!("\n\x1b[31m󰅙\x1b[0m Cancelled. Restored original colors.");
}
