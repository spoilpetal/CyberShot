use crate::{capture, config, platform};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::text::{Line, Span};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

fn terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(io::stdout()))?)
}

pub fn capture_tui() -> Result<()> {
    let mut term = terminal()?;
    let result = (|| -> Result<()> {
        let mut stage = 0u8; // 0 workspaces, 1 workspace contents, 2 window mode, 3 settings
        let mut selected = 0usize;
        let mut ws_selected = 0usize;
        let mut ws_id = 0i64;
        let mut window_selected = 0usize;
        let mut edit_dir: Option<String> = None;
        let mut msg = String::from("Choose a workspace   ↑/↓ Navigate   ENTER Select   ESC Exit");

        loop {
            let work = match platform::workspaces() {
                Ok(items) => items,
                Err(error) => {
                    if stage == 0 {
                        msg = format!("Workspace discovery unavailable: {error}");
                    }
                    vec![]
                }
            };
            let wins = if stage == 1 || stage == 2 {
                match platform::windows(ws_id) {
                    Ok(items) => items,
                    Err(error) => {
                        msg = format!("Window discovery failed: {error}");
                        vec![]
                    }
                }
            } else {
                vec![]
            };
            let settings = config::load()?;

            let items: Vec<ListItem> = match stage {
                0 => work
                    .iter()
                    .map(|w| {
                        ListItem::new(format!(
                            "  {}  {}  [{} windows]{}",
                            w.id,
                            w.name,
                            w.windows,
                            if w.active { "  • active" } else { "" }
                        ))
                    })
                    .collect(),
                1 => {
                    let mut choices = vec![ListItem::new("  Capture entire workspace")];
                    choices.extend(
                        wins.iter()
                            .map(|w| ListItem::new(format!("  {}  /  {}", w.title, w.app))),
                    );
                    choices
                }
                2 => vec![
                    ListItem::new("  Full window (include compositor borders)"),
                    ListItem::new("  Window content (exclude compositor borders)"),
                    ListItem::new("  Page area (select webpage only; exclude browser UI)"),
                ],
                _ => vec![],
            };
            if !items.is_empty() {
                selected = selected.min(items.len() - 1);
            } else {
                selected = 0;
            }

            term.draw(|f| {
                let outer = Layout::default().direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(6), Constraint::Length(3)])
                    .split(f.area());
                let heading = match stage {
                    0 => "CYBERSHOT / SELECT WORKSPACE",
                    1 => "CYBERSHOT / SELECT WINDOW OR WORKSPACE",
                    2 => "CYBERSHOT / WINDOW CAPTURE MODE",
                    _ => "CYBERSHOT / SETTINGS",
                };
                f.render_widget(Paragraph::new(heading).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))), outer[0]);
                let body = Layout::default().direction(Direction::Horizontal)
                    .constraints([Constraint::Length(30), Constraint::Min(25)]).split(outer[1]);
                let sidebar = match stage {
                    0 => format!("WORKSPACES\n\nPlatform: {}\n\nChoose the desktop workspace whose windows you want to browse.\n\nS  Settings", platform::operating_system()),
                    1 => {
                        let name = work.iter().find(|w| w.id == ws_id).map(|w| w.name.as_str()).unwrap_or("Selected workspace");
                        format!("WORKSPACE\n\n{}\nID: {}\nWindows: {}\n\nChoose the whole workspace or an individual window.", name, ws_id, wins.len())
                    }
                    2 => {
                        let title = wins.get(window_selected).map(|w| w.title.as_str()).unwrap_or("Selected window");
                        format!("WINDOW\n\n{}\n\nFull window includes compositor borders.\n\nWindow content excludes compositor borders.\n\nPage area opens a frozen-frame selector: draw around the webpage itself, leaving tabs and address bar outside.", title)
                    }
                    _ => format!("SETTINGS\n\nPlatform: {}\n\nScreenshot folder\n{}\n\nD  Edit path\nESC returns", platform::operating_system(), config::screenshot_dir().map(|p| p.display().to_string()).unwrap_or_default()),
                };
                f.render_widget(Paragraph::new(sidebar).style(Style::default().fg(Color::Gray))
                    .block(Block::default().title(" SYSTEM ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray))), body[0]);
                let title = match stage { 0 => " WORKSPACES ", 1 => " WORKSPACE CONTENTS ", 2 => " CAPTURE MODE ", _ => " SETTINGS " };
                if stage == 3 {
                    let value = edit_dir.as_deref().unwrap_or_else(|| settings.screenshot_dir.as_ref().and_then(|p| p.to_str()).unwrap_or("Default Pictures/CyberShot"));
                    f.render_widget(Paragraph::new(format!("SCREENSHOT DIRECTORY\n\n{}\n\nD  Edit path\nENTER  Save path\nESC  Cancel / Back" , value))
                        .block(Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))), body[1]);
                } else {
                    let mut state = ListState::default();
                    if !items.is_empty() { state.select(Some(selected)); }
                    f.render_stateful_widget(List::new(items).block(Block::default().title(title).borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)))
                        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
                        .highlight_symbol("  > "), body[1], &mut state);
                }
                let footer = match stage {
                    0 => " ↑/↓ Navigate   ENTER Choose workspace   s Settings   ESC Exit ",
                    1 => " ↑/↓ Navigate   ENTER Select   BACKSPACE Back ",
                    2 => " ↑/↓ Navigate   ENTER Capture   BACKSPACE Back ",
                    _ => " d Edit folder   ENTER Save   ESC Back ",
                };
                f.render_widget(Paragraph::new(format!("{}  |  {}", footer, msg)).style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL)), outer[2]);
            })?;

            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Esc if stage == 3 && edit_dir.is_some() => {
                        edit_dir = None;
                        msg = "Folder edit cancelled".into();
                    }
                    KeyCode::Esc if stage == 3 => {
                        stage = 0;
                        selected = ws_selected;
                        msg = "Returned to workspaces".into();
                    }
                    KeyCode::Esc => break,
                    KeyCode::Char(c)
                        if stage == 3 && edit_dir.is_some() && !c.is_control() && c != '\0' =>
                    {
                        if let Some(value) = edit_dir.as_mut() {
                            value.push(c);
                        }
                    }
                    KeyCode::Backspace if stage == 3 && edit_dir.is_some() => {
                        if let Some(value) = edit_dir.as_mut() {
                            value.pop();
                        }
                    }
                    KeyCode::Enter if stage == 3 && edit_dir.is_some() => {
                        let value = edit_dir.take().unwrap_or_default();
                        if value.trim().is_empty() {
                            msg = "Folder path cannot be empty".into();
                        } else {
                            let mut cfg = config::load()?;
                            cfg.screenshot_dir = Some(PathBuf::from(value.trim()));
                            config::save(&cfg)?;
                            msg = "Screenshot folder saved".into();
                        }
                    }
                    KeyCode::Char('d') if stage == 3 && edit_dir.is_none() => {
                        edit_dir = Some(config::screenshot_dir()?.display().to_string());
                        msg = "Type the new path; ENTER saves".into();
                    }
                    KeyCode::Char('s') if stage == 0 => {
                        stage = 3;
                        selected = 0;
                        msg = "Settings opened".into();
                    }
                    KeyCode::Backspace | KeyCode::Left if stage == 2 => {
                        stage = 1;
                        selected = window_selected + 1;
                        msg = "Choose another window".into();
                    }
                    KeyCode::Backspace | KeyCode::Left if stage == 1 => {
                        stage = 0;
                        selected = ws_selected;
                        msg = "Choose a workspace".into();
                    }
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => selected = selected.saturating_add(1),
                    KeyCode::Enter if stage == 0 => {
                        if let Some(w) = work.get(selected) {
                            ws_selected = selected;
                            ws_id = w.id;
                            let mut cfg = config::load()?;
                            cfg.selected_workspace = Some(ws_id);
                            config::save(&cfg)?;
                            stage = 1;
                            selected = 0;
                            msg = "Choose the workspace or one of its windows".into();
                        }
                    }
                    KeyCode::Enter if stage == 1 => {
                        if selected == 0 {
                            capture::workspace(ws_id)?;
                            msg = "Workspace captured".into();
                            break;
                        }
                        if selected <= wins.len() {
                            window_selected = selected - 1;
                            stage = 2;
                            selected = 0;
                            msg = "Choose full window, compositor content, or an exact page-area crop".into();
                        }
                    }
                    KeyCode::Enter if stage == 2 => {
                        let mode = match selected {
                            0 => capture::WindowCaptureMode::Full,
                            1 => capture::WindowCaptureMode::Content,
                            _ => capture::WindowCaptureMode::Page,
                        };
                        capture::window_by_index(ws_id, window_selected + 1, mode)?;
                        msg = "Window captured".into();
                        break;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    result
}
fn file_icon(path: &Path) -> &'static str {
    if path.is_dir() {
        return "📁";
    }
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "tiff" => "🖼️",
        "mp4" | "mkv" | "mov" | "avi" | "webm" => "🎞️",
        "mp3" | "wav" | "flac" | "ogg" => "🎵",
        "pdf" | "doc" | "docx" | "odt" | "txt" | "md" => "📄",
        "zip" | "tar" | "gz" | "7z" | "rar" => "📦",
        "rs" | "py" | "js" | "ts" | "c" | "cpp" | "toml" | "json" | "sh" => "💻",
        _ => "📃",
    }
}

/// Render an image as true-colour half-blocks. Works in Kitty and most modern
/// terminals without relying on a terminal-specific graphics protocol.
fn image_preview(path: &Path, width: u16, height: u16) -> Vec<Line<'static>> {
    let w = width.saturating_sub(2).max(1) as u32;
    let h = height.saturating_sub(2).max(1) as u32 * 2;
    let Ok(image) = image::open(path) else {
        return vec![Line::from(Span::raw("Preview unavailable for this file"))];
    };
    let image = image
        .resize(w, h, image::imageops::FilterType::Triangle)
        .to_rgb8();
    let mut lines = Vec::new();
    for y in (0..image.height()).step_by(2) {
        let mut spans = Vec::new();
        for x in 0..image.width() {
            let top = image.get_pixel(x, y).0;
            let bottom = if y + 1 < image.height() {
                image.get_pixel(x, y + 1).0
            } else {
                [0, 0, 0]
            };
            spans.push(Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(top[0], top[1], top[2]))
                    .bg(Color::Rgb(bottom[0], bottom[1], bottom[2])),
            ));
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// Kitty can render the original PNG at terminal pixel resolution instead of
/// reducing it to the terminal's character-cell resolution.
fn kitty_graphics_supported() -> bool {
    std::env::var_os("KITTY_WINDOW_ID").is_some()
        || std::env::var("TERM").is_ok_and(|term| term == "xterm-kitty")
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let a = chunk[0] as u32;
        let b = chunk.get(1).copied().unwrap_or(0) as u32;
        let c = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (a << 16) | (b << 8) | c;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Transmit a PNG through Kitty's graphics protocol and place it in the preview pane.
/// The image is scaled by Kitty to the available cell rectangle at full terminal resolution.
fn kitty_show_image(path: &Path, columns: u16, rows: u16) -> Result<()> {
    use std::io::Write;
    let decoded = image::open(path)?;
    let (pixel_width, pixel_height) = (decoded.width(), decoded.height());
    let mut png = std::io::Cursor::new(Vec::new());
    decoded.write_to(&mut png, image::ImageFormat::Png)?;
    let encoded = base64_encode(png.get_ref());
    let mut stdout = io::stdout();
    // Image starts inside the bordered preview pane (row 4, column 2).
    stdout.write_all(b"\x1b[4;2H")?;
    let chunks = encoded.as_bytes().chunks(3072);
    let count = chunks.len();
    for (index, chunk) in chunks.enumerate() {
        let more = if index + 1 < count { 1 } else { 0 };
        if index == 0 {
            write!(
                stdout,
                "\x1b_Ga=T,f=100,s={pixel_width},v={pixel_height},c={columns},r={rows},i=4242,q=2,m={more};"
            )?;
        } else {
            write!(stdout, "\x1b_Gm={more};")?;
        }
        stdout.write_all(chunk)?;
        stdout.write_all(b"\x1b\\")?;
    }
    stdout.flush()?;
    Ok(())
}

fn kitty_clear_image() -> Result<()> {
    use std::io::Write;
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b_Ga=d,d=i,i=4242,q=2\x1b\\")?;
    stdout.flush()?;
    Ok(())
}

/// CyberShot's standalone two-pane file manager, styled independently from CyberClip.
pub fn file_manager_tui() -> Result<()> {
    let mut term = terminal()?;
    let result = (|| -> Result<()> {
        let mut current = std::env::current_dir()?;
        let mut selected = 0usize;
        let mut search = String::new();
        let mut searching = false;
        let mut clipboard: Vec<PathBuf> = Vec::new();
        let mut marked: Vec<PathBuf> = Vec::new();
        let mut pending_delete: Option<PathBuf> = None;
        let mut rename: Option<String> = None;
        // Decode only on explicit Enter, then reuse the rendered preview while open.
        let mut preview: Option<(PathBuf, Vec<Line<'static>>)> = None;
        let mut message = String::from(
            "Enter open · Backspace parent · Space mark · c copy · Ctrl+V paste · r rename · d delete",
        );
        loop {
            let mut entries = fs::read_dir(&current)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect::<Vec<_>>();
            entries.sort_by(|a, b| {
                b.is_dir().cmp(&a.is_dir()).then_with(|| {
                    a.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase()
                        .cmp(
                            &b.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_lowercase(),
                        )
                })
            });
            if !search.is_empty() {
                let q = search.to_lowercase();
                entries.retain(|p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&q)
                });
            }
            selected = if entries.is_empty() {
                0
            } else {
                selected.min(entries.len() - 1)
            };
            term.draw(|f| {
                if let Some((path, lines)) = &preview {
                    let area = f.area();
                    let popup = Layout::default().direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)])
                        .split(area);
                    f.render_widget(Paragraph::new(format!(" CYBERSHOT // IMAGE PREVIEW · {} ", path.file_name().unwrap_or_default().to_string_lossy()))
                        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                        .block(Block::default().borders(Borders::ALL)), popup[0]);
                    f.render_widget(Paragraph::new(lines.clone()).block(Block::default().title(" IMAGE ").borders(Borders::ALL)), popup[1]);
                    f.render_widget(Paragraph::new("ENTER / ESC  Back to files")
                        .style(Style::default().fg(Color::Yellow)).block(Block::default().borders(Borders::ALL)), popup[2]);
                    return;
                }
                let areas=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Min(7),Constraint::Length(3)]).split(f.area());
                f.render_widget(Paragraph::new(format!(" CYBERSHOT // FILE MANAGER  ·  {} ",current.display())).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).block(Block::default().borders(Borders::ALL)),areas[0]);
                let panels=Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(48),Constraint::Percentage(52)]).split(areas[1]);
                let items=entries.iter().map(|p| { let name=p.file_name().unwrap_or_default().to_string_lossy(); let mark=if marked.contains(p){" [MARKED]"}else{""}; ListItem::new(format!(" {}  {}{}",file_icon(p),name,mark)) }).collect::<Vec<_>>();
                let mut state=ListState::default(); if !items.is_empty(){state.select(Some(selected));}
                f.render_stateful_widget(List::new(items).block(Block::default().title(format!(" DIRECTORY · {} ITEMS ",entries.len())).borders(Borders::ALL)).highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)).highlight_symbol(" > "),panels[0],&mut state);
                let detail_area=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(9),Constraint::Min(4)]).split(panels[1]);
                let detail=if let Some(path)=entries.get(selected){let size=fs::metadata(path).map(|m|m.len()).unwrap_or(0);format!("{}  NAME\n{}\n\nTYPE  {}  ·  SIZE  {} bytes\nMARKED  {}  ·  CLIPBOARD  {}\n\n{}",file_icon(path),path.file_name().unwrap_or_default().to_string_lossy(),if path.is_dir(){"Directory"}else{"File"},size,marked.len(),clipboard.len(),path.display())}else{"This directory is empty".into()};
                f.render_widget(Paragraph::new(detail).block(Block::default().title(" ITEM DETAILS ").borders(Borders::ALL)),detail_area[0]);
                f.render_widget(Paragraph::new("Press ENTER on an image to preview")
                    .block(Block::default().title(" PREVIEW ").borders(Borders::ALL)),detail_area[1]);
                let footer=if let Some(path)=&pending_delete{format!("DELETE {}?  Y confirm  N cancel",path.display())}else if let Some(name)=&rename{format!("RENAME TO: {}  ENTER save  ESC cancel",name)}else if searching{format!("SEARCH: {}  ENTER/Esc finish",search)}else{format!("{}  |  / Search  |  f native folder  |  Q Quit",message)};
                f.render_widget(Paragraph::new(footer).style(Style::default().fg(Color::Yellow)).block(Block::default().borders(Borders::ALL)),areas[2]);
            })?;
            if let Some((path, lines)) = &preview {
                if lines.is_empty() && kitty_graphics_supported() {
                    let size = term.size()?;
                    kitty_show_image(
                        path,
                        size.width.saturating_sub(4),
                        size.height.saturating_sub(6),
                    )?;
                }
            }
            if let Event::Key(k) = event::read()? {
                if preview.is_some() {
                    if matches!(k.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                        if kitty_graphics_supported() {
                            kitty_clear_image()?;
                        }
                        preview = None;
                    }
                    continue;
                }
                if let Some(path) = pending_delete.take() {
                    match k.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            let metadata = fs::symlink_metadata(&path)?;
                            if metadata.file_type().is_symlink() || metadata.is_file() {
                                fs::remove_file(&path)?;
                            } else if metadata.is_dir() {
                                fs::remove_dir_all(&path)?;
                            }
                            marked.retain(|p| p != &path);
                            message = "Deleted item".into();
                        }
                        _ => message = "Delete cancelled".into(),
                    }
                    continue;
                }
                if let Some(buffer) = rename.as_mut() {
                    match k.code {
                        KeyCode::Esc => {
                            rename = None;
                            message = "Rename cancelled".into();
                        }
                        KeyCode::Backspace => {
                            buffer.pop();
                        }
                        KeyCode::Enter => {
                            if let Some(old) = entries.get(selected) {
                                let name = buffer.trim();
                                if name.is_empty() || name.contains('/') || name.contains('\\') {
                                    message = "Invalid filename".into();
                                } else {
                                    let dest = current.join(name);
                                    if dest.exists() {
                                        message = "That name already exists".into();
                                    } else {
                                        fs::rename(old, dest)?;
                                        message = "Renamed item".into();
                                    }
                                }
                            }
                            rename = None;
                        }
                        KeyCode::Char(c) if !c.is_control() => buffer.push(c),
                        _ => {}
                    }
                    continue;
                }
                if searching {
                    match k.code {
                        KeyCode::Esc | KeyCode::Enter => searching = false,
                        KeyCode::Backspace => {
                            search.pop();
                            selected = 0;
                        }
                        KeyCode::Char(c) if !c.is_control() => {
                            search.push(c);
                            selected = 0;
                        }
                        _ => {}
                    }
                    continue;
                }
                if k.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(k.code, KeyCode::Char('v') | KeyCode::Char('V'))
                {
                    let mut count = 0;
                    for source in &clipboard {
                        if source.is_dir() && current.starts_with(source) {
                            message = format!(
                                "Cannot paste {} into itself or a child directory",
                                source.display()
                            );
                            continue;
                        }
                        let name = source.file_name().unwrap_or_default();
                        let mut dest = current.join(name);
                        if dest.exists() {
                            let stem = source.file_stem().unwrap_or_default().to_string_lossy();
                            let ext = source
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|e| format!(".{e}"))
                                .unwrap_or_default();
                            let mut n = 1;
                            loop {
                                dest = current.join(format!("{stem}-copy-{n}{ext}"));
                                if !dest.exists() {
                                    break;
                                }
                                n += 1;
                            }
                        }
                        copy_item(source, &dest)?;
                        count += 1;
                    }
                    message = format!("Pasted {count} item(s)");
                    continue;
                }
                match k.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => {
                        if selected + 1 < entries.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::PageUp => selected = selected.saturating_sub(10),
                    KeyCode::PageDown => {
                        selected = (selected + 10).min(entries.len().saturating_sub(1))
                    }
                    KeyCode::Enter => {
                        if let Some(path) = entries.get(selected) {
                            if path.is_dir() {
                                current = path.clone();
                                selected = 0;
                                search.clear();
                            } else if is_image(path) {
                                let size = term.size()?;
                                let w = size.width.saturating_sub(4);
                                let h = size.height.saturating_sub(6);
                                preview = Some((
                                    path.clone(),
                                    if kitty_graphics_supported() {
                                        Vec::new()
                                    } else {
                                        image_preview(path, w, h)
                                    },
                                ));
                            } else {
                                platform::open_file(path)?;
                                message = "Opened file in default application".into();
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(parent) = current.parent() {
                            current = parent.to_path_buf();
                            selected = 0;
                            search.clear();
                        }
                    }
                    KeyCode::Char('/') => {
                        search.clear();
                        searching = true;
                    }
                    KeyCode::Char(' ') => {
                        if let Some(path) = entries.get(selected) {
                            if marked.contains(path) {
                                marked.retain(|p| p != path);
                            } else {
                                marked.push(path.clone());
                            }
                        }
                    }
                    KeyCode::Char('c') => {
                        clipboard = if marked.is_empty() {
                            entries.get(selected).cloned().into_iter().collect()
                        } else {
                            marked.clone()
                        };
                        message = format!(
                            "Copied {} item(s) to CyberShot file clipboard",
                            clipboard.len()
                        );
                    }
                    KeyCode::Char('r') => {
                        if let Some(path) = entries.get(selected) {
                            rename = Some(
                                path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned(),
                            );
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(path) = entries.get(selected) {
                            pending_delete = Some(path.clone());
                        }
                    }
                    KeyCode::Char('f') => {
                        platform::open_file_manager(&current)?;
                        message = "Opened native file manager".into();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    result
}

fn copy_item(source: &std::path::Path, destination: &std::path::Path) -> Result<()> {
    if fs::symlink_metadata(source)?.file_type().is_symlink() {
        anyhow::bail!("Copying symbolic links is not supported by the built-in file clipboard");
    }
    if source.is_dir() {
        fs::create_dir(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_item(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else {
        fs::copy(source, destination)?;
    }
    Ok(())
}

/// Browse, open, rename, delete, and copy captured screenshots.
pub fn history_tui() -> Result<()> {
    let mut term = terminal()?;
    let result = (|| -> Result<()> {
        let dir = config::screenshot_dir()?;
        let mut selected = 0usize;
        let mut msg = String::from("↑/↓ Select  ENTER View  C Copy  R Rename  D Delete  ESC Quit");
        let mut rename: Option<String> = None;
        // Cache the decoded terminal preview; selection changes never decode images.
        let mut preview: Option<(PathBuf, Vec<Line<'static>>)> = None;

        loop {
            let entries = read_images(&dir)?;
            selected = if entries.is_empty() {
                0
            } else {
                selected.min(entries.len() - 1)
            };
            term.draw(|f| {
                if let Some((path, lines)) = &preview {
                    let area = f.area();
                    let popup = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Min(1),
                            Constraint::Length(3),
                        ])
                        .split(area);
                    f.render_widget(
                        Paragraph::new(format!(
                            " CYBERSHOT // IMAGE PREVIEW · {} ",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        ))
                        .style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                        .block(Block::default().borders(Borders::ALL)),
                        popup[0],
                    );
                    f.render_widget(
                        Paragraph::new(lines.clone())
                            .block(Block::default().title(" IMAGE ").borders(Borders::ALL)),
                        popup[1],
                    );
                    f.render_widget(
                        Paragraph::new("ENTER / ESC  Back to history")
                            .style(Style::default().fg(Color::Yellow))
                            .block(Block::default().borders(Borders::ALL)),
                        popup[2],
                    );
                    return;
                }
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ])
                    .split(f.area());
                f.render_widget(
                    Paragraph::new(format!(
                        " CYBERSHOT // CAPTURE HISTORY  ·  {} items ",
                        entries.len()
                    ))
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(Block::default().borders(Borders::ALL)),
                    chunks[0],
                );
                let items: Vec<ListItem> = entries
                    .iter()
                    .map(|p| {
                        let name = p.file_name().unwrap_or_default().to_string_lossy();
                        let size = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                        ListItem::new(format!(
                            "  {}  {name}   ({:.1} KB)",
                            file_icon(p),
                            size as f64 / 1024.0
                        ))
                    })
                    .collect();
                let mut state = ListState::default();
                if !items.is_empty() {
                    state.select(Some(selected));
                }
                let body = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
                    .split(chunks[1]);
                f.render_stateful_widget(
                    List::new(items)
                        .block(
                            Block::default()
                                .title(" RECENT CAPTURES ")
                                .borders(Borders::ALL),
                        )
                        .highlight_style(
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol("  > "),
                    body[0],
                    &mut state,
                );
                f.render_widget(
                    Paragraph::new("Press ENTER to preview this capture")
                        .block(Block::default().title(" PREVIEW ").borders(Borders::ALL)),
                    body[1],
                );
                let footer = if let Some(value) = &rename {
                    format!("Rename to: {value}  ·  ENTER confirm  ·  ESC cancel")
                } else {
                    msg.clone()
                };
                f.render_widget(
                    Paragraph::new(footer)
                        .style(Style::default().fg(Color::Yellow))
                        .block(Block::default().borders(Borders::ALL)),
                    chunks[2],
                );
            })?;
            if let Some((path, lines)) = &preview {
                if lines.is_empty() && kitty_graphics_supported() {
                    let size = term.size()?;
                    kitty_show_image(
                        path,
                        size.width.saturating_sub(4),
                        size.height.saturating_sub(6),
                    )?;
                }
            }

            if let Event::Key(k) = event::read()? {
                if preview.is_some() {
                    if matches!(k.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
                        if kitty_graphics_supported() {
                            kitty_clear_image()?;
                        }
                        preview = None;
                    }
                    continue;
                }
                if let Some(buffer) = rename.as_mut() {
                    match k.code {
                        KeyCode::Esc => {
                            rename = None;
                            msg = "Rename cancelled".into();
                        }
                        KeyCode::Enter => {
                            if let Some(old) = entries.get(selected) {
                                let clean = buffer.trim();
                                if clean.is_empty() {
                                    msg = "Filename cannot be empty".into();
                                } else {
                                    let mut filename = clean.to_string();
                                    if std::path::Path::new(&filename).extension().is_none() {
                                        filename.push_str(
                                            old.extension()
                                                .and_then(|e| e.to_str())
                                                .map(|e| format!(".{e}"))
                                                .unwrap_or_default()
                                                .as_str(),
                                        );
                                    }
                                    let destination = dir.join(filename);
                                    if destination.exists() {
                                        msg = "A file with that name already exists".into();
                                    } else {
                                        fs::rename(old, destination)?;
                                        msg = "Screenshot renamed".into();
                                    }
                                }
                            }
                            rename = None;
                        }
                        KeyCode::Backspace => {
                            buffer.pop();
                        }
                        KeyCode::Char(c) if !c.is_control() && c != '/' && c != '\\' => {
                            buffer.push(c)
                        }
                        _ => {}
                    }
                    continue;
                }
                match k.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => selected = selected.saturating_add(1),
                    KeyCode::Enter => {
                        if let Some(path) = entries.get(selected) {
                            let size = term.size()?;
                            preview = Some((
                                path.clone(),
                                if kitty_graphics_supported() {
                                    Vec::new()
                                } else {
                                    image_preview(
                                        path,
                                        size.width.saturating_sub(4),
                                        size.height.saturating_sub(6),
                                    )
                                },
                            ));
                        }
                    }
                    KeyCode::Char('o') => {
                        if let Some(path) = entries.get(selected) {
                            platform::open_file(path)?;
                            msg = format!(
                                "Opened {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                        }
                    }
                    KeyCode::Char('c') => {
                        if let Some(path) = entries.get(selected) {
                            let image = image::load_from_memory(&fs::read(path)?)?;
                            let mut png = std::io::Cursor::new(Vec::new());
                            image.write_to(&mut png, image::ImageFormat::Png)?;
                            crate::clipboard::set_image(png.get_ref())?;
                            msg = "Screenshot copied to clipboard".into();
                        }
                    }
                    KeyCode::Char('r') => {
                        if let Some(path) = entries.get(selected) {
                            rename = Some(
                                path.file_stem()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned(),
                            );
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(path) = entries.get(selected) {
                            fs::remove_file(path)?;
                            selected = selected.saturating_sub(1);
                            msg = "Screenshot deleted".into();
                        }
                    }
                    KeyCode::Char('f') => {
                        platform::open_file_manager(&dir)?;
                        msg = "Opened screenshot folder".into();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    result
}

fn is_image(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "tiff"
        )
    })
}

fn read_images(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                matches!(
                    e.to_ascii_lowercase().as_str(),
                    "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "tiff"
                )
            })
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
    files.reverse();
    Ok(files)
}
