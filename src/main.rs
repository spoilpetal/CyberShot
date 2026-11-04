mod app;
mod capture;
mod clipboard;
mod config;
#[cfg(target_os = "linux")]
mod overlay;
mod platform;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cybershot",
    version,
    about = "Terminal-first screenshot utility"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// List workspaces or windows in the selected workspace.
    Show { target: String },

    /// Select a workspace by its displayed number.
    Sl { target: String, number: usize },

    /// Capture current, full screen, a region, a workspace, or a window.
    Capture {
        target: String,
        number: Option<usize>,
        #[arg(long = "noh")]
        no_header: bool,
        #[arg(long)]
        workspace: Option<i64>,
        #[arg(long)]
        window: Option<usize>,
    },

    /// Open CyberShot's two-pane file manager.
    Files,

    /// Browse captured screenshots, open them, rename, delete, or copy to clipboard.
    History,

    /// Open the capture/settings TUI.
    Tui,

    /// Open the frozen-frame region selector.
    Select,

    /// Print the screenshot directory.
    Location,

    /// Enable, disable, or inspect the external keyboard-shortcut trigger.
    Keybind {
        #[command(subcommand)]
        action: KeybindAction,
    },

    /// Region-capture entry point for a shortcut configured in your OS/desktop.
    Hotkey,

    /// Internal clipboard provider.
    #[command(name = "__clipboard-serve", hide = true)]
    ClipboardServe { payload: PathBuf, ready: PathBuf },
}

#[derive(Subcommand)]
enum KeybindAction {
    /// Enable the `csb hotkey` trigger.
    Enable {
        /// Optional label for the shortcut configured in your desktop.
        #[arg(long)]
        shortcut: Option<String>,
    },
    /// Disable the `csb hotkey` trigger.
    Disable,
    /// Show the trigger state.
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Command::Tui) => tui::capture_tui()?,
        Some(Command::Files) => tui::file_manager_tui()?,
        Some(Command::History) => tui::history_tui()?,
        Some(Command::Select) => {
            #[cfg(target_os = "linux")]
            {
                if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                    overlay::run()?;
                } else {
                    capture::region()?;
                }
            }
            #[cfg(not(target_os = "linux"))]
            capture::region()?;
        }
        Some(Command::Location) => {
            println!("{}", config::screenshot_dir()?.display());
        }
        Some(Command::Keybind { action }) => {
            let mut cfg = config::load()?;
            match action {
                KeybindAction::Enable { shortcut } => {
                    if let Some(value) = shortcut {
                        let value = value.trim();
                        if value.is_empty() {
                            anyhow::bail!("Shortcut label cannot be empty");
                        }
                        cfg.keybind = Some(value.to_string());
                    }
                    cfg.keybind_enabled = true;
                    config::save(&cfg)?;
                    println!("CyberShot shortcut trigger enabled.");
                    println!("Assign your OS/desktop shortcut to run: csb hotkey");
                    println!(
                        "Shortcut label: {}",
                        cfg.keybind.as_deref().unwrap_or("not set")
                    );
                }
                KeybindAction::Disable => {
                    cfg.keybind_enabled = false;
                    config::save(&cfg)?;
                    println!(
                        "CyberShot shortcut trigger disabled. The OS shortcut can remain configured safely."
                    );
                }
                KeybindAction::Status => {
                    println!(
                        "Shortcut trigger: {}",
                        if cfg.keybind_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    );
                    println!(
                        "Shortcut label: {}",
                        cfg.keybind.as_deref().unwrap_or("not set")
                    );
                    println!("OS/desktop shortcut command: csb hotkey");
                }
            }
        }
        Some(Command::Hotkey) => {
            if config::load()?.keybind_enabled {
                capture::region()?;
            } else {
                println!(
                    "CyberShot shortcut trigger is disabled. Run `csb keybind enable` to turn it on."
                );
            }
        }
        Some(Command::ClipboardServe { payload, ready }) => {
            clipboard::serve(&payload, &ready)?;
        }

        Some(Command::Show { target }) => match target.as_str() {
            "work" | "workspace" | "workspaces" => {
                for (i, workspace) in platform::workspaces()?.iter().enumerate() {
                    println!(
                        "{:>2}. {}  [{} windows]{}",
                        i + 1,
                        workspace.name,
                        workspace.windows,
                        if workspace.active { "  * active" } else { "" }
                    );
                }
            }
            "wind" | "window" | "windows" => {
                let selected_workspace = config::load()?.selected_workspace.ok_or_else(|| {
                    anyhow::anyhow!("No workspace selected. Run `csb sl work <number>` first.")
                })?;

                for (i, window) in platform::windows(selected_workspace)?.iter().enumerate() {
                    println!("{:>2}. {}  ({})", i + 1, window.title, window.app);
                }
            }
            _ => anyhow::bail!("Use `csb show work` or `csb show wind`"),
        },

        Some(Command::Sl { target, number }) => {
            if target != "work" && target != "workspace" {
                anyhow::bail!("Use `csb sl work <number>`");
            }

            let workspaces = platform::workspaces()?;
            let chosen = workspaces
                .get(
                    number
                        .checked_sub(1)
                        .ok_or_else(|| anyhow::anyhow!("Numbers start at 1"))?,
                )
                .ok_or_else(|| anyhow::anyhow!("Workspace number {number} does not exist"))?;

            let mut cfg = config::load()?;
            cfg.selected_workspace = Some(chosen.id);
            config::save(&cfg)?;

            println!("Selected workspace {} ({})", number, chosen.name);
        }

        Some(Command::Capture {
            target,
            number,
            no_header,
            workspace,
            window,
        }) => {
            if let Some(workspace_id) = workspace {
                let window_number =
                    window.ok_or_else(|| anyhow::anyhow!("--workspace requires --window"))?;
                capture::window(workspace_id, window_number, no_header)?;
            } else {
                match target.as_str() {
                    "work" | "workspace" => {
                        // Explicit number takes priority over the saved selection.
                        // Numbering follows `csb show work`.
                        let workspace_id = if let Some(number) = number {
                            let workspaces = platform::workspaces()?;
                            let chosen = workspaces
                                .get(number.checked_sub(1).ok_or_else(|| {
                                    anyhow::anyhow!("Workspace numbers start at 1")
                                })?)
                                .ok_or_else(|| {
                                    anyhow::anyhow!("Workspace number {number} does not exist")
                                })?;
                            chosen.id
                        } else {
                            config::load()?.selected_workspace.ok_or_else(|| {
                                anyhow::anyhow!("Select a workspace first: `csb sl work <number>`")
                            })?
                        };

                        capture::workspace(workspace_id)?;
                    }
                    "wind" | "window" => {
                        let workspace_id = config::load()?.selected_workspace.ok_or_else(|| {
                            anyhow::anyhow!("Select a workspace first: `csb sl work <number>`")
                        })?;
                        let window_number = number.ok_or_else(|| {
                            anyhow::anyhow!("Usage: csb capture wind <number> [-noh]")
                        })?;

                        capture::window_by_index(workspace_id, window_number, no_header)?;
                    }
                    "current" => capture::current()?,
                    "full" => capture::full()?,
                    "region" => capture::region()?,
                    _ => anyhow::bail!("Use work, wind, current, full, or region"),
                }
            }
        }
    }

    Ok(())
}
