mod app;
mod capture;
mod clipboard;
mod config;
#[cfg(target_os = "linux")]
mod overlay;
mod platform;
mod tui;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cybershot",
    version,
    disable_help_subcommand = true,
    about = "Workspace-first screenshot utility",
    long_about = "CyberShot captures screenshots on Linux, macOS, and Windows. Use the interactive TUI for keyboard-driven capture, or run a capture command directly from your terminal. Captures are saved in your configured screenshot directory and copied to the clipboard when supported.",
    after_help = "QUICK START:\n  cybershot                         Open the capture TUI\n  cybershot show work               List workspaces\n  cybershot show wind --workspace 1  List windows in workspace 1\n  cybershot capture workspace 1     Capture workspace 1\n  cybershot capture window 1 --workspace 1 --mode content\n                                    Capture window 1 without compositor borders\n  cybershot capture window 1 --workspace 1 --mode page\n                                    Interactively select only the webpage viewport\n  cybershot history                 Browse previous screenshots\n\nRun `cybershot help <COMMAND>` for detailed command help."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// List available workspaces or windows in the selected workspace.
    #[command(
        long_about = "List desktop workspaces with `show work`, or list windows from a workspace with `show wind --workspace ID`. If --workspace is omitted for windows, CyberShot uses the workspace saved by `sl work NUMBER`."
    )]
    Show {
        /// What to list: work (workspaces) or wind (windows in a workspace).
        #[arg(value_name = "TARGET", help = "work | wind")]
        target: String,
        /// Workspace ID for `show wind`; defaults to the saved selection.
        #[arg(long)]
        workspace: Option<i64>,
    },

    /// Select a workspace by its displayed number.
    #[command(
        long_about = "Select a workspace using the number shown by `show work`. The selection is saved and is used by workspace/window capture commands that do not specify a workspace explicitly."
    )]
    Sl {
        /// Select a workspace (currently `work`).
        #[arg(value_name = "TARGET", help = "work")]
        target: String,
        /// Workspace number from `show work`.
        #[arg(value_name = "NUMBER")]
        number: usize,
    },

    /// Capture a full screen, selected region, workspace, or window.
    #[command(
        long_about = "Capture a workspace or a window directly. Use `capture workspace NUMBER` to capture the workspace at the displayed position from `show work`. Use `capture window NUMBER --workspace ID` to capture a window listed by `show wind --workspace ID`. Window mode is `full` (include compositor borders), `content` (exclude compositor borders), or `page` (open an interactive frozen-frame crop so you can select only a webpage viewport, excluding browser tabs and address bar). Browser chrome cannot be detected reliably from generic window geometry, so `page` asks you to draw the exact webpage area. The `full`, `region`, and `current` targets remain available for screen-level captures."
    )]
    Capture {
        /// Capture target: workspace/work, window/wind, full, region, or current.
        #[arg(
            value_name = "TARGET",
            help = "workspace | window | work | wind | full | region | current"
        )]
        target: String,
        /// Workspace/window number, depending on TARGET.
        #[arg(value_name = "NUMBER")]
        number: Option<usize>,
        /// Window capture mode: full includes compositor borders; content trims them.
        #[arg(long, value_enum, default_value = "full")]
        mode: WindowCaptureModeArg,
        /// Explicit workspace ID for window capture.
        #[arg(long, help = "Workspace ID containing the target window")]
        workspace: Option<i64>,
        /// Window number within the specified workspace (legacy alternative to TARGET NUMBER).
        #[arg(long, help = "Window number within --workspace")]
        window: Option<usize>,
        /// Deprecated compatibility flag; equivalent to --mode content.
        #[arg(long = "noh", hide = true)]
        no_header: bool,
    },

    /// Open CyberShot's two-pane file manager.
    #[command(
        long_about = "Browse the screenshot directory in a two-pane file manager. Use it to navigate folders and manage screenshot files."
    )]
    Files,

    /// Browse captured screenshots, open them, rename, delete, or copy to clipboard.
    #[command(
        long_about = "Open the screenshot history browser. Select a saved screenshot to open it, rename it, delete it, or copy it to the clipboard."
    )]
    History,

    /// Open the capture/settings TUI.
    #[command(
        long_about = "Open the interactive capture workflow: choose a workspace, choose the whole workspace or one of its windows, then choose full-window or content-only capture. Screenshot-folder settings are available from the workspace screen."
    )]
    Tui,

    /// Open the graphical region selector.
    #[command(
        long_about = "Freeze the desktop and let you draw a rectangle around the area to capture. This is the direct interactive region-selection command."
    )]
    Select,

    /// Print the screenshot directory.
    #[command(
        long_about = "Print the absolute path where CyberShot saves screenshots. The directory is created if it does not already exist."
    )]
    Location,

    /// Enable, disable, or inspect the external keyboard-shortcut trigger.
    #[command(
        long_about = "Manage the safety switch used by desktop shortcuts. When enabled, `hotkey` launches region capture; when disabled, shortcut invocations do nothing. This does not create or remove the desktop shortcut itself."
    )]
    Keybind {
        #[command(subcommand)]
        action: KeybindAction,
    },

    /// Region-capture entry point for a shortcut configured in your OS/desktop.
    #[command(
        long_about = "Run region capture when the keybind trigger is enabled. Intended as the command launched by Hyprland or another desktop's custom keyboard shortcut."
    )]
    Hotkey,

    /// Show detailed help for CyberShot or a specific command.
    #[command(
        long_about = "Display the full help page. Add a command name to see its detailed usage, arguments, and options. Nested commands are supported, for example `help keybind enable`."
    )]
    Help {
        /// Optional command path, such as `capture` or `keybind enable`.
        command: Vec<String>,
    },

    /// Internal clipboard provider.
    #[command(name = "__clipboard-serve", hide = true)]
    ClipboardServe { payload: PathBuf, ready: PathBuf },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum WindowCaptureModeArg {
    #[default]
    Full,
    Content,
    /// Interactively select the webpage viewport, excluding browser controls.
    Page,
}

impl From<WindowCaptureModeArg> for capture::WindowCaptureMode {
    fn from(value: WindowCaptureModeArg) -> Self {
        match value {
            WindowCaptureModeArg::Full => Self::Full,
            WindowCaptureModeArg::Content => Self::Content,
            WindowCaptureModeArg::Page => Self::Page,
        }
    }
}

#[derive(Subcommand)]
enum KeybindAction {
    /// Enable the desktop shortcut trigger.
    #[command(
        long_about = "Allow `hotkey` to launch the region selector. Optionally save a human-readable shortcut label, such as `SUPER SHIFT C`."
    )]
    Enable {
        /// Optional label for the shortcut configured in your desktop.
        #[arg(long)]
        shortcut: Option<String>,
    },
    /// Disable the desktop shortcut trigger without removing the OS-level binding.
    Disable,
    /// Show whether the desktop shortcut trigger is enabled and its label.
    Status,
}

fn command_name() -> String {
    std::env::args_os()
        .next()
        .and_then(|arg| {
            std::path::PathBuf::from(arg)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "cybershot".to_string())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Command::Tui) => tui::capture_tui()?,
        Some(Command::Help { command }) => {
            let mut root = Cli::command();
            let mut current = &mut root;
            for name in &command {
                current = current.find_subcommand_mut(name).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Unknown help topic `{name}`. Run `cybershot --help` to see commands."
                    )
                })?;
            }
            current.print_long_help()?;
            println!();
        }
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
                    println!(
                        "Assign your OS/desktop shortcut to run: {} hotkey",
                        command_name()
                    );
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
                    println!("OS/desktop shortcut command: {} hotkey", command_name());
                }
            }
        }
        Some(Command::Hotkey) => {
            if config::load()?.keybind_enabled {
                capture::hotkey()?;
            } else {
                println!(
                    "CyberShot shortcut trigger is disabled. Run `{} keybind enable` to turn it on.",
                    command_name()
                );
            }
        }
        Some(Command::ClipboardServe { payload, ready }) => {
            clipboard::serve(&payload, &ready)?;
        }

        Some(Command::Show { target, workspace }) => match target.as_str() {
            "work" | "workspace" | "workspaces" => {
                for (i, workspace) in platform::workspaces()?.iter().enumerate() {
                    println!(
                        "{:>2}. {}  (ID: {})  [{} windows]{}",
                        i + 1,
                        workspace.name,
                        workspace.id,
                        workspace.windows,
                        if workspace.active { "  * active" } else { "" }
                    );
                }
            }
            "wind" | "window" | "windows" => {
                let selected_workspace = match workspace.or(config::load()?.selected_workspace) {
                    Some(id) => id,
                    None => anyhow::bail!(
                        "Specify --workspace ID or select one with `cybershot sl work <number>`"
                    ),
                };

                for (i, window) in platform::windows(selected_workspace)?.iter().enumerate() {
                    println!("{:>2}. {}  ({})", i + 1, window.title, window.app);
                }
            }
            _ => {
                anyhow::bail!("Use `cybershot show work` or `cybershot show wind [--workspace ID]`")
            }
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
            mode,
            workspace,
            window,
            no_header,
        }) => {
            let capture_mode = if no_header {
                capture::WindowCaptureMode::Content
            } else {
                mode.into()
            };
            if let Some(workspace_id) = workspace {
                let window_number = window.or(number).ok_or_else(|| anyhow::anyhow!("Usage: cybershot capture window NUMBER --workspace ID [--mode full|content]"))?;
                capture::window(workspace_id, window_number, capture_mode)?;
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
                            anyhow::anyhow!("Usage: cybershot capture window <number> [--workspace ID] [--mode full|content]")
                        })?;

                        capture::window_by_index(workspace_id, window_number, capture_mode)?;
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
