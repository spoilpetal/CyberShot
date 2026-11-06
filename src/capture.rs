use crate::{clipboard, config, platform};
use anyhow::{Context, Result};
use chrono::Local;
use std::{path::PathBuf, thread, time::Duration};

fn output_path(label: &str) -> Result<PathBuf> {
    let dir = config::screenshot_dir()?;
    Ok(dir.join(format!(
        "{}-{}.png",
        Local::now().format("%Y%m%d-%H%M%S-%3f"),
        label
    )))
}

pub fn current() -> Result<()> {
    full()
}

pub fn full() -> Result<()> {
    let path = output_path("screen")?;
    platform::capture_full(&path)?;
    println!("Saved: {}", path.display());
    copy_image(&path)
}

/// Entry point used by desktop shortcuts.
pub fn hotkey() -> Result<()> {
    region()
}

pub fn region() -> Result<()> {
    #[cfg(target_os = "linux")]
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        crate::overlay::run()?;
        return Ok(());
    }

    let path = output_path("region")?;
    platform::capture_region(&path)?;
    println!("Saved: {}", path.display());
    copy_image(&path)
}

/// Capture a workspace/virtual desktop.
pub fn workspace(id: i64) -> Result<()> {
    let active = platform::active_workspace()?;

    if active != id {
        platform::switch_workspace(id)?;
        thread::sleep(Duration::from_millis(350));
    }

    let path = output_path(&format!("workspace-{id}"))?;
    let result = platform::capture_full(&path);

    if active != id {
        let _ = platform::switch_workspace(active);
    }

    result?;
    println!("Saved: {}", path.display());
    copy_image(&path)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowCaptureMode {
    Full,
    Content,
    /// Interactive crop for application content such as a browser webpage viewport.
    Page,
}

fn window_capture_rect(
    at: [i64; 2],
    size: [i64; 2],
    border_size: i64,
    mode: WindowCaptureMode,
) -> Option<(i64, i64, i64, i64)> {
    // Reject invalid window dimensions before calculating borders.
    if size[0] <= 0 || size[1] <= 0 {
        return None;
    }

    let border = border_size.max(0);

    let rect = match mode {
        WindowCaptureMode::Full => {
            let twice_border = border.checked_mul(2)?;

            (
                at[0].checked_sub(border)?,
                at[1].checked_sub(border)?,
                size[0].checked_add(twice_border)?,
                size[1].checked_add(twice_border)?,
            )
        }
        WindowCaptureMode::Content => (at[0], at[1], size[0], size[1]),
        // Page mode is handled by the interactive frozen-frame selector in
        // window_geometry; it has no fixed window rectangle.
        WindowCaptureMode::Page => return None,
    };

    (rect.2 > 0 && rect.3 > 0).then_some(rect)
}

pub fn window_by_index(workspace: i64, index: usize, mode: WindowCaptureMode) -> Result<()> {
    let list = platform::windows(workspace)?;

    let win = list
        .get(index.checked_sub(1).context("Window numbers start at 1")?)
        .ok_or_else(|| anyhow::anyhow!("Window {index} not found in workspace {workspace}"))?;

    window_geometry(
        win.at,
        win.size,
        win.border_size,
        workspace,
        &win.title,
        mode,
    )
}

pub fn window(workspace: i64, index: usize, mode: WindowCaptureMode) -> Result<()> {
    window_by_index(workspace, index, mode)
}

fn window_geometry(
    at: [i64; 2],
    size: [i64; 2],
    border_size: i64,
    workspace: i64,
    title: &str,
    mode: WindowCaptureMode,
) -> Result<()> {
    // Validate geometry before changing the active workspace.
    let rect = if mode == WindowCaptureMode::Page {
        None
    } else {
        Some(
            window_capture_rect(at, size, border_size, mode)
                .ok_or_else(|| anyhow::anyhow!("Window has invalid capture dimensions"))?,
        )
    };

    let active = platform::active_workspace()?;

    if active != workspace {
        platform::switch_workspace(workspace)?;
        thread::sleep(Duration::from_millis(350));
    }

    let result = if let Some((x, y, width, height)) = rect {
        let path = output_path(if mode == WindowCaptureMode::Content {
            "window-content"
        } else {
            "window-full"
        })?;
        platform::capture_rect(x, y, width, height, &path).and_then(|()| {
            println!("Saved: {}", path.display());
            copy_image(&path)
        })
    } else {
        // Browser chrome is application UI, not compositor decoration. It cannot
        // be identified reliably from generic window geometry, so let the user
        // select the exact webpage viewport on the frozen desktop frame.
        println!("Select only the webpage viewport; exclude tabs and address bar.");
        region()
    };

    if active != workspace {
        let _ = platform::switch_workspace(active);
    }

    result.with_context(|| format!("Could not capture window: {title}"))
}

#[cfg(test)]
mod tests {
    use super::{WindowCaptureMode, window_capture_rect};

    #[test]
    fn full_window_expands_to_include_compositor_border() {
        assert_eq!(
            window_capture_rect([100, 200], [800, 600], 2, WindowCaptureMode::Full),
            Some((98, 198, 804, 604))
        );
    }

    #[test]
    fn content_mode_uses_application_surface_bounds() {
        assert_eq!(
            window_capture_rect([100, 200], [800, 600], 2, WindowCaptureMode::Content),
            Some((100, 200, 800, 600))
        );
    }

    #[test]
    fn page_mode_has_no_fixed_window_rectangle() {
        assert_eq!(
            window_capture_rect([100, 200], [800, 600], 2, WindowCaptureMode::Page),
            None
        );
    }

    #[test]
    fn invalid_window_dimensions_are_rejected() {
        assert_eq!(
            window_capture_rect([0, 0], [0, 100], 2, WindowCaptureMode::Full),
            None
        );
    }

    #[test]
    fn negative_window_dimensions_are_rejected() {
        assert_eq!(
            window_capture_rect([0, 0], [-10, 100], 2, WindowCaptureMode::Full),
            None
        );
    }

    #[test]
    fn overflowing_border_calculation_is_rejected() {
        assert_eq!(
            window_capture_rect([0, 0], [100, 100], i64::MAX, WindowCaptureMode::Full),
            None
        );
    }
}

fn copy_image(path: &PathBuf) -> Result<()> {
    let bytes = std::fs::read(path)?;
    clipboard::set_image(&bytes)?;
    println!("Copied screenshot to clipboard.");
    Ok(())
}
