use super::super::{Window, Workspace};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
struct WsJson {
    id: i64,
    name: String,
    #[serde(default)]
    windows: usize,
}
#[derive(Deserialize)]
struct ClientJson {
    #[serde(default)]
    title: String,
    #[serde(default, rename = "class")]
    app: String,
    at: [i64; 2],
    size: [i64; 2],
    workspace: WsRef,
}
#[derive(Deserialize)]
struct WsRef {
    id: i64,
}

fn hypr(args: &[&str]) -> Result<String> {
    let out = Command::new("hyprctl")
        .args(args)
        .output()
        .context("Could not execute hyprctl; is Hyprland running?")?;
    if !out.status.success() {
        anyhow::bail!(
            "hyprctl {:?} failed (status {}): {}{}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
    }
    Ok(String::from_utf8(out.stdout)?)
}
pub fn workspaces() -> Result<Vec<Workspace>> {
    let raw = hypr(&["-j", "workspaces"])?;
    let items: Vec<WsJson> = serde_json::from_str(&raw)?;
    let active: i64 =
        serde_json::from_str::<serde_json::Value>(&hypr(&["-j", "activeworkspace"])?)?["id"]
            .as_i64()
            .unwrap_or(-999);
    let mut out: Vec<_> = items
        .into_iter()
        .map(|w| Workspace {
            id: w.id,
            name: w.name,
            windows: w.windows,
            active: w.id == active,
        })
        .collect();
    out.sort_by_key(|w| w.id);
    Ok(out)
}
pub fn windows(workspace: i64) -> Result<Vec<Window>> {
    let raw = hypr(&["-j", "clients"])?;
    let clients: Vec<ClientJson> = serde_json::from_str(&raw)?;
    Ok(clients
        .into_iter()
        .filter(|c| c.workspace.id == workspace)
        .map(|c| Window {
            title: c.title,
            app: c.app,
            at: c.at,
            size: c.size,
        })
        .collect())
}
/// Switch workspaces across both Hyprland config providers. Hyprland 0.54+
/// Lua configs require dispatcher objects; legacy hyprlang configs still use
/// the traditional `dispatch workspace <id>` form.
pub fn switch_workspace(id: i64) -> Result<()> {
    if uses_lua_config_provider() {
        // `id` is an integer, so it is safe to interpolate into this Lua string.
        let expression = format!("hl.dsp.focus({{ workspace = \"{id}\" }})");
        hypr(&["dispatch", &expression])?;
    } else {
        let workspace = id.to_string();
        hypr(&["dispatch", "workspace", &workspace])?;
    }
    Ok(())
}

/// Detect the active config provider at runtime rather than assuming a
/// particular Hyprland version or config style. If `systeminfo` is unavailable
/// (older releases), retain the legacy dispatch behavior.
fn uses_lua_config_provider() -> bool {
    let Ok(output) = Command::new("hyprctl").arg("systeminfo").output() else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().split_once(':'))
        .any(|(key, value)| {
            key.trim().eq_ignore_ascii_case("configProvider")
                && value.trim().eq_ignore_ascii_case("lua")
        })
}
pub fn active_workspace() -> Result<i64> {
    let v: serde_json::Value = serde_json::from_str(&hypr(&["-j", "activeworkspace"])?)?;
    v["id"]
        .as_i64()
        .context("Could not read active workspace id")
}
