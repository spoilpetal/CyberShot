//! Platform clipboard adapter. Linux Wayland keeps CyberShot's self-hosted
//! wl-clipboard-rs provider; X11, Windows and macOS use their native clipboard
//! utilities. CyberShot never depends on CyberClip.

#[cfg(target_os = "linux")]
mod linux {
    //! Standalone native Wayland image clipboard provider.
    //! CyberShot does not require CyberClip or an external clipboard executable.
    use anyhow::{Context, Result, bail};
    use std::{fs, path::{Path,PathBuf}, process::{Command,Stdio}, thread, time::{Duration,Instant,SystemTime,UNIX_EPOCH}};
    use wl_clipboard_rs::copy::{MimeType,Options,Source};

    fn runtime_dir()->Result<PathBuf>{
        let base=std::env::var_os("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR is not set")?;
        Ok(PathBuf::from(base).join("cybershot"))
    }
    pub fn set_image(data:&[u8])->Result<()>{
        if std::env::var_os("WAYLAND_DISPLAY").is_none() {
            let mut child = Command::new("xclip")
                .args(["-selection", "clipboard", "-t", "image/png", "-i"])
                .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped())
                .spawn().context("On X11, install xclip to copy screenshots")?;
            use std::io::Write;
            child.stdin.take().context("Could not open xclip stdin")?.write_all(data)?;
            let status = child.wait()?;
            if !status.success() { bail!("xclip failed with status {status}"); }
            return Ok(());
        }
        let dir=runtime_dir()?;fs::create_dir_all(&dir)?;
        #[cfg(unix)] {use std::os::unix::fs::PermissionsExt;fs::set_permissions(&dir,fs::Permissions::from_mode(0o700))?;}
        let stamp=SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let payload=dir.join(format!("{}-{stamp}.png",std::process::id()));
        let ready=dir.join(format!("{}-{stamp}.ready",std::process::id()));
        #[cfg(unix)] {
            use std::fs::OpenOptions;use std::io::Write;use std::os::unix::fs::OpenOptionsExt;
            let mut f=OpenOptions::new().write(true).create_new(true).mode(0o600).open(&payload)?;
            f.write_all(data)?;
        }
        #[cfg(not(unix))] fs::write(&payload,data)?;
        let exe=std::env::current_exe()?;
        let mut child=Command::new(exe).arg("__clipboard-serve").arg(&payload).arg(&ready)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
        let deadline=Instant::now()+Duration::from_secs(3);
        loop{
            if ready.exists(){let _=fs::remove_file(ready);return Ok(());}
            if let Some(status)=child.try_wait()?{let _=fs::remove_file(payload);bail!("Clipboard provider exited early: {status}");}
            if Instant::now()>=deadline{let _=child.kill();let _=child.wait();let _=fs::remove_file(payload);bail!("Timed out starting clipboard provider");}
            thread::sleep(Duration::from_millis(20));
        }
    }
    pub fn serve(payload:&Path,ready:&Path)->Result<()>{
        let data=fs::read(payload).context("Cannot read clipboard image")?;let _=fs::remove_file(payload);
        let mut opts=Options::new();opts.foreground(true);
        let prepared=opts.prepare_copy(Source::Bytes(data.into()),MimeType::Specific("image/png".into()))
            .context("Cannot prepare Wayland image clipboard offer")?;
        fs::write(ready,b"ready")?;
        prepared.serve().context("Wayland clipboard provider failed")?;
        let _=fs::remove_file(ready);Ok(())
    }

}

#[cfg(target_os = "linux")]
pub use linux::{serve, set_image};

#[cfg(not(target_os = "linux"))]
pub fn serve(_payload: &std::path::Path, _ready: &std::path::Path) -> anyhow::Result<()> {
    anyhow::bail!("The detached clipboard provider is only used by the Linux Wayland backend")
}

#[cfg(target_os = "macos")]
pub fn set_image(data: &[u8]) -> anyhow::Result<()> {
    use anyhow::{Context, bail};
    use std::{fs, process::Command};
    let path = std::env::temp_dir().join(format!("cybershot-clipboard-{}.png", std::process::id()));
    fs::write(&path, data)?;
    let script = format!("set the clipboard to (read (POSIX file {:?}) as «class PNGf»)", path.to_string_lossy());
    let out = Command::new("osascript").arg("-e").arg(script).output().context("Could not run osascript")?;
    let _ = fs::remove_file(path);
    if !out.status.success() { bail!("macOS clipboard failed: {}", String::from_utf8_lossy(&out.stderr)); }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn set_image(data: &[u8]) -> anyhow::Result<()> {
    use anyhow::{Context, bail};
    use std::{fs, process::Command};
    let path = std::env::temp_dir().join(format!("cybershot-clipboard-{}.png", std::process::id()));
    fs::write(&path, data)?;
    let escaped = path.to_string_lossy().replace("'", "''");
    let script = format!("Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.Drawing; $i=[System.Drawing.Image]::FromFile('{}'); [System.Windows.Forms.Clipboard]::SetImage($i); $i.Dispose()", escaped);
    let out = Command::new("powershell.exe").args(["-NoProfile", "-STA", "-Command"]).arg(&script).output().context("Could not run PowerShell clipboard helper")?;
    let _ = fs::remove_file(path);
    if !out.status.success() { bail!("Windows clipboard failed: {}", String::from_utf8_lossy(&out.stderr)); }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn set_image(_data: &[u8]) -> anyhow::Result<()> {
    anyhow::bail!("Image clipboard is not implemented for this operating system")
}
