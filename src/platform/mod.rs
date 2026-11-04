//! Native desktop integration for Linux (Hyprland/X11), macOS and Windows.
//! Shared capture/TUI code calls this module instead of desktop-specific APIs.

use anyhow::{Context, Result};
use std::{path::Path, process::Command};

#[derive(Clone, Debug)]
pub struct Workspace { pub id: i64, pub name: String, pub windows: usize, pub active: bool }
#[derive(Clone, Debug)]
pub struct Window { pub title: String, pub app: String, pub at: [i64; 2], pub size: [i64; 2] }

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
fn is_hyprland() -> bool { std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() }

pub fn workspaces() -> Result<Vec<Workspace>> {
    #[cfg(target_os = "linux")]
    {
        if is_hyprland() { return linux::hyprland::workspaces(); }
        if std::env::var_os("DISPLAY").is_some() { return x11_workspaces(); }
        anyhow::bail!("Workspace discovery on this Wayland compositor is not available yet; full and region capture still work")
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    { Ok(vec![Workspace { id: 1, name: "Current desktop".into(), windows: windows(1).map(|items| items.len()).unwrap_or(0), active: true }]) }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    { anyhow::bail!("Desktop discovery is not supported on this operating system") }
}

pub fn windows(workspace: i64) -> Result<Vec<Window>> {
    #[cfg(target_os = "linux")]
    {
        if is_hyprland() { return linux::hyprland::windows(workspace); }
        if std::env::var_os("DISPLAY").is_some() { return x11_windows(workspace); }
        anyhow::bail!("Window discovery is unavailable for this Wayland compositor")
    }
    #[cfg(target_os = "windows")]
    { windows_windows() }
    #[cfg(target_os = "macos")]
    { windows_macos() }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    { let _ = workspace; anyhow::bail!("Window discovery is not supported on this operating system") }
}


pub fn switch_workspace(id: i64) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if is_hyprland() { return linux::hyprland::switch_workspace(id); }
        if std::env::var_os("DISPLAY").is_some() {
            let status = Command::new("wmctrl").args(["-s", &id.to_string()]).status().context("Install wmctrl to switch X11 workspaces")?;
            if !status.success() { anyhow::bail!("wmctrl could not switch to desktop {id}"); }
            return Ok(());
        }
        anyhow::bail!("Workspace switching is not exposed by this Wayland compositor")
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    { let _ = id; anyhow::bail!("Native virtual-desktop switching is not exposed by this backend; capture the current desktop instead") }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    { let _ = id; anyhow::bail!("Workspace switching is unsupported") }
}

pub fn active_workspace() -> Result<i64> {
    #[cfg(target_os = "linux")]
    {
        if is_hyprland() { return linux::hyprland::active_workspace(); }
        if std::env::var_os("DISPLAY").is_some() {
            let out = Command::new("wmctrl").arg("-d").output().context("Install wmctrl to inspect X11 desktops")?;
            let output = String::from_utf8_lossy(&out.stdout);
            for line in output.lines() { let fields: Vec<_> = line.split_whitespace().collect(); if fields.get(1) == Some(&"*") { return fields[0].parse().context("Invalid X11 desktop id"); } }
            anyhow::bail!("Could not determine the active X11 desktop")
        }
        anyhow::bail!("Active workspace detection is unavailable for this Wayland compositor")
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    { Ok(1) }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    { anyhow::bail!("Active workspace detection is unsupported") }
}

#[cfg(target_os = "linux")]
fn x11_workspaces() -> Result<Vec<Workspace>> {
    let out = Command::new("wmctrl").arg("-d").output().context("Install wmctrl for X11 workspace support")?;
    if !out.status.success() { anyhow::bail!("wmctrl -d failed"); }
    let mut result = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let f: Vec<_> = line.split_whitespace().collect(); if f.len() < 3 { continue; }
        let id: i64 = f[0].parse().unwrap_or(0);
        let active = f[1] == "*";
        result.push(Workspace { id, name: f[2].to_string(), windows: x11_windows(id)?.len(), active });
    }
    Ok(result)
}

#[cfg(target_os = "linux")]
fn x11_windows(workspace: i64) -> Result<Vec<Window>> {
    let out = Command::new("wmctrl").args(["-lpG"]).output().context("Install wmctrl for X11 window discovery")?;
    if !out.status.success() { anyhow::bail!("wmctrl -lpG failed"); }
    let mut result = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 9 { continue; }
        let desktop: i64 = fields[1].parse().unwrap_or(-1);
        if desktop != workspace { continue; }
        let x: i64 = fields[3].parse().unwrap_or(0);
        let y: i64 = fields[4].parse().unwrap_or(0);
        let w: i64 = fields[5].parse().unwrap_or(0);
        let h: i64 = fields[6].parse().unwrap_or(0);
        let title = fields[8..].join(" ");
        result.push(Window { title, app: "X11 window".into(), at: [x, y], size: [w, h] });
    }
    Ok(result)
}

#[cfg(target_os = "windows")]
fn windows_windows() -> Result<Vec<Window>> {
    let script = r#"$src=@'
using System; using System.Text; using System.Runtime.InteropServices;
public class WinList { public delegate bool EnumProc(IntPtr h, IntPtr l); [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr l); [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h); [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n); [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r); [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; } }
'@; Add-Type $src; $a=@(); [WinList]::EnumWindows({ param($h,$l) if([WinList]::IsWindowVisible($h)){ $s=New-Object Text.StringBuilder 1024; [void][WinList]::GetWindowText($h,$s,$s.Capacity); $r=New-Object WinList+RECT; if([WinList]::GetWindowRect($h,[ref]$r) -and $s.Length -gt 0){$a += [pscustomobject]@{address=$h.ToInt64().ToString();title=$s.ToString();app='Windows';x=$r.L;y=$r.T;w=($r.R-$r.L);h=($r.B-$r.T);workspace=1}} }; return $true },[IntPtr]::Zero) | Out-Null; ConvertTo-Json -Compress -InputObject @($a)"#;
    let out = Command::new("powershell.exe").args(["-NoProfile", "-Command", script]).output().context("Could not enumerate Windows desktop windows")?;
    if !out.status.success() { anyhow::bail!("Windows window enumeration failed: {}", String::from_utf8_lossy(&out.stderr)); }
    let values: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let mut result = Vec::new();
    for v in values.as_array().into_iter().flatten() { result.push(Window { title:v["title"].as_str().unwrap_or("").into(), app:"Windows".into(), at:[v["x"].as_i64().unwrap_or(0),v["y"].as_i64().unwrap_or(0)], size:[v["w"].as_i64().unwrap_or(0),v["h"].as_i64().unwrap_or(0)] }); }
    Ok(result)
}

#[cfg(target_os = "macos")]
fn windows_macos() -> Result<Vec<Window>> {
    let script = r#"tell application "System Events"
set output to ""
repeat with p in (every process whose background only is false)
try
repeat with w in (every window of p)
set pos to position of w
set sz to size of w
set output to output & (unix id of p) & "|" & (name of p) & "|" & (name of w) & "|" & (item 1 of pos) & "|" & (item 2 of pos) & "|" & (item 1 of sz) & "|" & (item 2 of sz) & linefeed
end repeat
end try
end repeat
end tell
return output"#;
    let out = Command::new("osascript").args(["-e", script]).output().context("Could not enumerate macOS windows; grant Accessibility permission to Terminal/CyberShot")?;
    if !out.status.success() { anyhow::bail!("macOS window enumeration failed (Accessibility permission may be required): {}", String::from_utf8_lossy(&out.stderr)); }
    let mut result = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() { let f: Vec<_> = line.splitn(7, '|').collect(); if f.len()!=7 { continue; }
        let n=|i:usize| f[i].trim().parse::<i64>().unwrap_or(0);
        result.push(Window { app:f[1].into(), title:f[2].into(), at:[n(3),n(4)], size:[n(5),n(6)] });
    }
    Ok(result)
}

/// Capture a rectangle in virtual-desktop coordinates.
pub fn capture_rect(x:i64,y:i64,w:i64,h:i64,path:&Path)->Result<()> {
    if w <= 0 || h <= 0 { anyhow::bail!("Capture rectangle has invalid dimensions"); }
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            let geometry=format!("{x},{y} {w}x{h}"); let out=Command::new("grim").args(["-g",&geometry]).arg(path).output().context("Install grim for Wayland capture")?;
            if !out.status.success(){anyhow::bail!("grim failed: {}",String::from_utf8_lossy(&out.stderr));}
        } else {
            let geometry=format!("{w}x{h}+{x}+{y}"); let out=Command::new("import").args(["-window","root","-crop",&geometry]).arg(path).output().context("Install ImageMagick for X11 capture")?;
            if !out.status.success(){anyhow::bail!("ImageMagick failed: {}",String::from_utf8_lossy(&out.stderr));}
        }
    }
    #[cfg(target_os = "macos")]
    { let rect=format!("{x},{y},{w},{h}"); let out=Command::new("screencapture").args(["-x","-R",&rect]).arg(path).output().context("Could not run screencapture")?; if !out.status.success(){anyhow::bail!("screencapture rectangle failed: {}",String::from_utf8_lossy(&out.stderr));} }
    #[cfg(target_os = "windows")]
    { let script=format!("Add-Type -AssemblyName System.Drawing; Add-Type -AssemblyName System.Windows.Forms; $b=[System.Windows.Forms.SystemInformation]::VirtualScreen; $bmp=New-Object System.Drawing.Bitmap({w},{h}); $g=[System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen({x},{y},0,0,$bmp.Size); $bmp.Save('{}',[System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $bmp.Dispose()",path.to_string_lossy().replace("'","''")); let out=Command::new("powershell.exe").args(["-NoProfile","-STA","-Command"]).arg(script).output().context("Could not run Windows rectangle capture")?; if !out.status.success(){anyhow::bail!("Windows rectangle capture failed: {}",String::from_utf8_lossy(&out.stderr));} }
    Ok(())
}

pub fn capture_full(path:&Path)->Result<()> {
    #[cfg(target_os="linux")] { if std::env::var_os("WAYLAND_DISPLAY").is_some(){let o=Command::new("grim").arg(path).output().context("Install grim for Wayland capture")?;if !o.status.success(){anyhow::bail!("grim failed: {}",String::from_utf8_lossy(&o.stderr));}}else{let o=Command::new("import").args(["-window","root"]).arg(path).output().context("Install ImageMagick for X11 capture")?;if !o.status.success(){anyhow::bail!("ImageMagick failed: {}",String::from_utf8_lossy(&o.stderr));}} }
    #[cfg(target_os="macos")] { let o=Command::new("screencapture").arg("-x").arg(path).output().context("Could not run screencapture")?;if !o.status.success(){anyhow::bail!("screencapture failed: {}",String::from_utf8_lossy(&o.stderr));} }
    #[cfg(target_os="windows")] { let script=format!("Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.Drawing; $b=[System.Windows.Forms.SystemInformation]::VirtualScreen; $i=New-Object System.Drawing.Bitmap($b.Width,$b.Height); $g=[System.Drawing.Graphics]::FromImage($i); $g.CopyFromScreen($b.Location,[System.Drawing.Point]::Empty,$b.Size); $i.Save('{}',[System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $i.Dispose()",path.to_string_lossy().replace("'","''")); let o=Command::new("powershell.exe").args(["-NoProfile","-STA","-Command"]).arg(script).output().context("Could not run Windows screenshot helper")?;if !o.status.success(){anyhow::bail!("Windows screenshot failed: {}",String::from_utf8_lossy(&o.stderr));} }
    Ok(())
}

pub fn capture_region(path:&Path)->Result<()> {
    #[cfg(target_os="linux")] { if std::env::var_os("WAYLAND_DISPLAY").is_some(){let s=Command::new("slurp").output().context("Install slurp for region selection")?;if !s.status.success(){anyhow::bail!("Region selection cancelled");}let g=String::from_utf8(s.stdout)?.trim().to_string();let o=Command::new("grim").args(["-g",&g]).arg(path).output()?;if !o.status.success(){anyhow::bail!("grim region capture failed");}}else{let s=Command::new("slop").args(["-f","%g"]).output().context("Install slop for X11 region selection")?;if !s.status.success(){anyhow::bail!("Region selection cancelled");}let g=String::from_utf8(s.stdout)?.trim().to_string();let o=Command::new("import").args(["-window","root","-crop",&g]).arg(path).output()?;if !o.status.success(){anyhow::bail!("ImageMagick region capture failed");}} }
    #[cfg(target_os="macos")] { let o=Command::new("screencapture").args(["-i"]).arg(path).output().context("Could not run screencapture")?;if !o.status.success(){anyhow::bail!("Region capture cancelled or failed");} }
    #[cfg(target_os="windows")] { let escaped=path.to_string_lossy().replace("'","''"); let script=format!(r#"Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.Drawing; $b=[System.Windows.Forms.SystemInformation]::VirtualScreen; $f=New-Object System.Windows.Forms.Form; $f.FormBorderStyle='None'; $f.StartPosition='Manual'; $f.Location=New-Object System.Drawing.Point($b.X,$b.Y); $f.Size=New-Object System.Drawing.Size($b.Width,$b.Height); $f.TopMost=$true; $f.ShowInTaskbar=$false; $f.BackColor='Black'; $f.Opacity=.25; $f.Cursor=[System.Windows.Forms.Cursors]::Cross; $script:p=$null; $script:r=[System.Drawing.Rectangle]::Empty; $f.Add_MouseDown({{param($s,$e) $script:p=$e.Location}}); $f.Add_MouseMove({{param($s,$e) if($script:p){{$x=[Math]::Min($script:p.X,$e.X);$y=[Math]::Min($script:p.Y,$e.Y);$script:r=[System.Drawing.Rectangle]::new($x,$y,[Math]::Abs($e.X-$script:p.X),[Math]::Abs($e.Y-$script:p.Y));$f.Invalidate()}}}}); $f.Add_MouseUp({{param($s,$e) if($script:r.Width -gt 2 -and $script:r.Height -gt 2){{$f.Tag='ok';$f.Close()}}}}); $f.Add_Paint({{param($s,$e) if($script:r.Width -gt 0){{$e.Graphics.DrawRectangle([System.Drawing.Pens]::Lime,$script:r)}}}}); [void]$f.ShowDialog(); if($f.Tag -ne 'ok'){{exit 2}}; $rect=[System.Drawing.Rectangle]::new($b.X+$script:r.X,$b.Y+$script:r.Y,$script:r.Width,$script:r.Height);$bmp=New-Object System.Drawing.Bitmap($rect.Width,$rect.Height);$g=[System.Drawing.Graphics]::FromImage($bmp);$g.CopyFromScreen($rect.Location,[System.Drawing.Point]::Empty,$rect.Size);$bmp.Save('{}',[System.Drawing.Imaging.ImageFormat]::Png);$g.Dispose();$bmp.Dispose()"#,escaped);let o=Command::new("powershell.exe").args(["-NoProfile","-STA","-ExecutionPolicy","Bypass","-Command"]).arg(script).output().context("Could not run Windows region selector")?;if !o.status.success(){anyhow::bail!("Region selection cancelled or failed: {}",String::from_utf8_lossy(&o.stderr));} }
    Ok(())
}

#[cfg(target_os="linux")] pub fn open_file(path:&Path)->Result<()>{Command::new("xdg-open").arg(path).spawn().context("Could not run xdg-open")?;Ok(())}
#[cfg(target_os="macos")] pub fn open_file(path:&Path)->Result<()>{Command::new("open").arg(path).spawn().context("Could not run open")?;Ok(())}
#[cfg(target_os="windows")] pub fn open_file(path:&Path)->Result<()>{Command::new("cmd").args(["/C","start",""]).arg(path).spawn().context("Could not open file")?;Ok(())}
#[cfg(target_os="linux")] pub fn open_file_manager(path:&Path)->Result<()>{open_file(path)}
#[cfg(target_os="macos")] pub fn open_file_manager(path:&Path)->Result<()>{open_file(path)}
#[cfg(target_os="windows")] pub fn open_file_manager(path:&Path)->Result<()>{Command::new("explorer.exe").arg(path).spawn().context("Could not open Explorer")?;Ok(())}

pub const fn operating_system()-> &'static str { if cfg!(target_os="linux"){"Linux"}else if cfg!(target_os="windows"){"Windows"}else if cfg!(target_os="macos"){"macOS"}else{"Other"} }

#[cfg(not(any(target_os="linux", target_os="macos", target_os="windows")))]
pub fn capture_rect(_:i64,_:i64,_:i64,_:i64,_:&Path)->Result<()> { anyhow::bail!("Rectangle capture is unsupported on this operating system") }
#[cfg(not(any(target_os="linux", target_os="macos", target_os="windows")))]
pub fn capture_full(_: &Path)->Result<()> { anyhow::bail!("Screen capture is unsupported on this operating system") }
#[cfg(not(any(target_os="linux", target_os="macos", target_os="windows")))]
pub fn capture_region(_: &Path)->Result<()> { anyhow::bail!("Region capture is unsupported on this operating system") }
#[cfg(not(any(target_os="linux", target_os="macos", target_os="windows")))]
pub fn open_file(_: &Path)->Result<()> { anyhow::bail!("Opening files is unsupported on this operating system") }
#[cfg(not(any(target_os="linux", target_os="macos", target_os="windows")))]
pub fn open_file_manager(_: &Path)->Result<()> { anyhow::bail!("Opening a native file manager is unsupported on this operating system") }
