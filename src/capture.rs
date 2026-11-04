use anyhow::{Context, Result};
use chrono::Local;
use std::{path::PathBuf, thread, time::Duration};
use crate::{config, platform, clipboard};

fn output_path(label: &str) -> Result<PathBuf> {
    let dir = config::screenshot_dir()?;
    Ok(dir.join(format!("{}-{}.png", Local::now().format("%Y%m%d-%H%M%S-%3f"), label)))
}

pub fn current() -> Result<()> { full() }

pub fn full() -> Result<()> {
    let path = output_path("screen")?;
    platform::capture_full(&path)?;
    println!("Saved: {}", path.display());
    copy_image(&path)
}

pub fn region() -> Result<()> {
    let path = output_path("region")?;
    platform::capture_region(&path)?;
    println!("Saved: {}", path.display());
    copy_image(&path)
}

/// Capture a workspace/virtual desktop. Hyprland and X11 switch to the
/// requested desktop temporarily; macOS and Windows expose the current desktop.
pub fn workspace(id: i64) -> Result<()> {
    let active = platform::active_workspace()?;
    if active != id {
        platform::switch_workspace(id)?;
        thread::sleep(Duration::from_millis(350));
    }
    let path = output_path(&format!("workspace-{id}"))?;
    let result = platform::capture_full(&path);
    if active != id { let _ = platform::switch_workspace(active); }
    result?;
    println!("Saved: {}", path.display());
    copy_image(&path)
}

pub fn window_by_index(workspace: i64, index: usize, no_header: bool) -> Result<()> {
    let list = platform::windows(workspace)?;
    let win = list.get(index.checked_sub(1).context("Window numbers start at 1")?)
        .ok_or_else(|| anyhow::anyhow!("Window {index} not found in workspace {workspace}"))?;
    window_geometry(win.at, win.size, workspace, &win.title, no_header)
}

pub fn window(workspace: i64, index: usize, no_header: bool) -> Result<()> {
    window_by_index(workspace, index, no_header)
}

fn window_geometry(at: [i64; 2], size: [i64; 2], workspace: i64, title: &str, no_header: bool) -> Result<()> {
    let active = platform::active_workspace()?;
    if active != workspace {
        platform::switch_workspace(workspace)?;
        thread::sleep(Duration::from_millis(350));
    }

    // `--noh` removes the top 32 px of the window frame/titlebar from the
    // capture rectangle. Clamp dimensions so tiny windows fail clearly.
    let header = if no_header { 32 } else { 0 };
    let height = size[1] - header;
    if height <= 0 { anyhow::bail!("Window is too small to crop its title bar"); }
    let path = output_path(if no_header { "window-no-header" } else { "window" })?;
    let result = platform::capture_rect(at[0], at[1] + header, size[0], height, &path);
    if active != workspace { let _ = platform::switch_workspace(active); }
    result.with_context(|| format!("Could not capture window: {title}"))?;
    println!("Saved: {}", path.display());
    copy_image(&path)
}

fn copy_image(path: &PathBuf) -> Result<()> {
    let bytes = std::fs::read(path)?;
    clipboard::set_image(&bytes)?;
    println!("Copied screenshot to clipboard.");
    Ok(())
}
