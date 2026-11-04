use anyhow::Result;
use crossterm::{event::{self,Event,KeyCode,KeyModifiers}, execute, terminal::{enable_raw_mode,disable_raw_mode,EnterAlternateScreen,LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, Terminal, layout::{Layout,Constraint,Direction}, widgets::{Block,Borders,List,ListItem,ListState,Paragraph}, style::{Style,Color,Modifier}};
use std::{io,fs,path::PathBuf};
use crate::{config,platform,capture};

fn terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?; execute!(io::stdout(),EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(io::stdout()))?)
}

pub fn capture_tui() -> Result<()> {
    let mut term = terminal()?;

    let result = (|| -> Result<()> {
        let mut stage = 0u8;
        let mut selected = 0usize;
        let mut ws_selected = 0usize;
        let mut ws_id = 0i64;
        let mut edit_dir: Option<String> = None;
        let mut msg = String::from(
            "UP/DOWN: Navigate   ENTER: Select   ESC: Exit"
        );

        loop {
            let work = platform::workspaces().unwrap_or_default();
            let generic_capture = work.is_empty();
            let wins = if stage == 1 {
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

            let items: Vec<ListItem> = if stage == 0 && generic_capture {
                vec![ListItem::new("  Full-screen capture"), ListItem::new("  Region capture")]
            } else if stage == 0 {
                work.iter()
                    .map(|w| {
                        ListItem::new(format!(
                            "  {}   {}   [{} windows]",
                            w.id, w.name, w.windows
                        ))
                    })
                    .collect()
            } else if stage == 1 {
                wins.iter()
                    .map(|w| ListItem::new(format!("  {}  /  {}", w.title, w.app)))
                    .collect()
            } else { vec![] };

            if !items.is_empty() {
                selected = selected.min(items.len() - 1);
            } else {
                selected = 0;
            }

            term.draw(|f| {
                let outer = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(6),
                        Constraint::Length(3),
                    ])
                    .split(f.area());

                f.render_widget(
                    Paragraph::new(if stage == 2 { "  CYBERSHOT  /  CAPTURE + SETTINGS" } else { "  CYBERSHOT  /  CAPTURE CONTROL" })
                        .style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(
                                    Style::default().fg(Color::Cyan)
                                ),
                        ),
                    outer[0],
                );

                let body = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(25),
                        Constraint::Min(25),
                    ])
                    .split(outer[1]);

                let sidebar_text = if stage == 2 {
                    format!("SETTINGS\n\nPlatform: {}\n\nScreenshot folder\n{}\n\nPress D to change folder\nESC returns", platform::operating_system(), config::screenshot_dir().map(|p| p.display().to_string()).unwrap_or_default())
                } else if stage == 0 && generic_capture {
                    format!(
                        "MODE\n\nQUICK CAPTURE\n\nPlatform: {}\n\nChoose full-screen\nor region capture.",
                        platform::operating_system()
                    )
                } else if stage == 0 {
                    format!(
                        "MODE\n\nWORKSPACE SELECT\n\nAvailable: {}\n\nChoose a workspace\nwith ENTER.",
                        work.len()
                    )
                } else {
                    let name = work
                        .iter()
                        .find(|w| w.id == ws_id)
                        .map(|w| w.name.as_str())
                        .unwrap_or("Unknown");

                    format!(
                        "MODE\n\nWINDOW SELECT\n\nWorkspace: {}\nID: {}\nWindows: {}\n\nBACKSPACE returns\nto workspaces.",
                        name,
                        ws_id,
                        wins.len()
                    )
                };

                f.render_widget(
                    Paragraph::new(sidebar_text)
                        .style(Style::default().fg(Color::Gray))
                        .block(
                            Block::default()
                                .title(" SYSTEM ")
                                .borders(Borders::ALL)
                                .border_style(
                                    Style::default().fg(Color::DarkGray)
                                ),
                        ),
                    body[0],
                );

                let title = if stage == 2 { " SETTINGS " } else if stage == 0 && generic_capture {
                    " QUICK CAPTURE "
                } else if stage == 0 {
                    " WORKSPACES "
                } else {
                    " WINDOWS "
                };

                let mut state = ListState::default();
                if !items.is_empty() {
                    state.select(Some(selected));
                }

                if stage == 2 {
                    let value = edit_dir.as_deref().unwrap_or_else(|| settings.screenshot_dir.as_ref().map(|p| p.to_str().unwrap_or("")).unwrap_or("Default Pictures/CyberShot"));
                    let text = format!("SCREENSHOT DIRECTORY\n\n{}\n\nD  Edit path\nENTER  Save path\nESC  Cancel / Back", value);
                    f.render_widget(Paragraph::new(text).block(Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))), body[1]);
                } else { f.render_stateful_widget(
                    List::new(items)
                        .block(
                            Block::default()
                                .title(title)
                                .borders(Borders::ALL)
                                .border_style(
                                    Style::default().fg(Color::Cyan)
                                ),
                        )
                        .highlight_style(
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol("  > "),
                    body[1],
                    &mut state,
                ); }

                let footer = if stage == 2 { " s Settings   d Edit folder   ENTER Save   ESC Back " } else if stage == 0 {
                    " ↑/↓ Navigate   ENTER Choose   s Settings   ESC Exit "
                } else {
                    " ↑/↓ Navigate   ENTER Capture   W Capture Workspace   BACKSPACE Back "
                };

                f.render_widget(
                    Paragraph::new(format!("{}  |  {}", footer, msg))
                        .style(Style::default().fg(Color::Yellow))
                        .block(Block::default().borders(Borders::ALL)),
                    outer[2],
                );
            })?;

            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Esc if stage == 2 && edit_dir.is_some() => { edit_dir = None; msg = "Folder edit cancelled".into(); }
                    KeyCode::Esc if stage == 2 => { stage = 0; msg = "Returned to capture".into(); }
                    KeyCode::Esc => break,

                    KeyCode::Char(c) if stage == 2 && edit_dir.is_some() && !c.is_control() && c != '\0' => { if let Some(value) = edit_dir.as_mut() { value.push(c); } }
                    KeyCode::Backspace if stage == 2 && edit_dir.is_some() => { if let Some(value) = edit_dir.as_mut() { value.pop(); } }
                    KeyCode::Enter if stage == 2 && edit_dir.is_some() => {
                        let value = edit_dir.take().unwrap_or_default();
                        if value.trim().is_empty() { msg = "Folder path cannot be empty".into(); }
                        else { let mut cfg = config::load()?; cfg.screenshot_dir = Some(PathBuf::from(value.trim())); config::save(&cfg)?; msg = "Screenshot folder saved".into(); }
                    }
                    KeyCode::Char('d') if stage == 2 && edit_dir.is_none() => {
                        edit_dir = Some(config::screenshot_dir()?.display().to_string());
                        msg = "Type the new path; ENTER saves".into();
                    }
                    KeyCode::Backspace if stage == 1 => {
                        stage = 0;
                        selected = ws_selected;
                        msg = String::from("Returned to workspace selection");
                    }

                    KeyCode::Left if stage == 1 => {
                        stage = 0;
                        selected = ws_selected;
                        msg = String::from("Returned to workspace selection");
                    }

                    KeyCode::Char('s') if stage == 0 => { stage = 2; selected = 0; msg = "Settings opened".into(); }
                    KeyCode::Up => {
                        selected = selected.saturating_sub(1);
                    }

                    KeyCode::Down => {
                        selected = selected.saturating_add(1);
                    }

                    KeyCode::Char('w') if stage == 1 => {
                        capture::workspace(ws_id)?;
                        msg = String::from("Workspace captured");
                        break;
                    }

                    KeyCode::Enter if stage == 0 => {
                        if generic_capture {
                            match selected {
                                0 => capture::full()?,
                                1 => capture::region()?,
                                _ => {}
                            }
                            break;
                        }
                        if let Some(w) = work.get(selected) {
                            ws_selected = selected;
                            ws_id = w.id;

                            let mut c = config::load()?;
                            c.selected_workspace = Some(ws_id);
                            config::save(&c)?;

                            stage = 1;
                            selected = 0;
                            msg = String::from("Select a window to capture");
                        }
                    }

                    KeyCode::Enter if stage == 1 => {
                        if wins.get(selected).is_some() {
                            capture::window_by_index(
                                ws_id,
                                selected + 1,
                                false,
                            )?;
                            msg = String::from("Window captured");
                            break;
                        }
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
        let mut message = String::from("Enter open · Backspace parent · Space mark · c copy · Ctrl+V paste · r rename · d delete");
        loop {
            let mut entries = fs::read_dir(&current)?.filter_map(|e| e.ok()).map(|e| e.path()).collect::<Vec<_>>();
            entries.sort_by(|a,b| b.is_dir().cmp(&a.is_dir()).then_with(|| a.file_name().unwrap_or_default().to_string_lossy().to_lowercase().cmp(&b.file_name().unwrap_or_default().to_string_lossy().to_lowercase())));
            if !search.is_empty() { let q=search.to_lowercase(); entries.retain(|p| p.file_name().unwrap_or_default().to_string_lossy().to_lowercase().contains(&q)); }
            selected = if entries.is_empty() { 0 } else { selected.min(entries.len()-1) };
            term.draw(|f| {
                let areas=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Min(7),Constraint::Length(3)]).split(f.area());
                f.render_widget(Paragraph::new(format!(" CYBERSHOT // FILE MANAGER  ·  {} ",current.display())).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).block(Block::default().borders(Borders::ALL)),areas[0]);
                let panels=Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(48),Constraint::Percentage(52)]).split(areas[1]);
                let items=entries.iter().map(|p| { let dir=p.is_dir(); let name=p.file_name().unwrap_or_default().to_string_lossy(); let mark=if marked.contains(p){" [MARKED]"}else{""}; ListItem::new(format!(" {}  {}{}",if dir{"DIR"}else{"FILE"},name,mark)) }).collect::<Vec<_>>();
                let mut state=ListState::default(); if !items.is_empty(){state.select(Some(selected));}
                f.render_stateful_widget(List::new(items).block(Block::default().title(format!(" DIRECTORY · {} ITEMS ",entries.len())).borders(Borders::ALL)).highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)).highlight_symbol(" > "),panels[0],&mut state);
                let detail=if let Some(path)=entries.get(selected){let size=fs::metadata(path).map(|m|m.len()).unwrap_or(0);format!("NAME\n{}\n\nTYPE\n{}\n\nSIZE\n{} bytes\n\nPATH\n{}\n\nMARKED\n{} item(s)\n\nCLIPBOARD\n{} item(s)",path.file_name().unwrap_or_default().to_string_lossy(),if path.is_dir(){"Directory"}else{"File"},size,path.display(),marked.len(),clipboard.len())}else{"This directory is empty".into()};
                f.render_widget(Paragraph::new(detail).block(Block::default().title(" ITEM DETAILS ").borders(Borders::ALL)),panels[1]);
                let footer=if let Some(path)=&pending_delete{format!("DELETE {}?  Y confirm  N cancel",path.display())}else if let Some(name)=&rename{format!("RENAME TO: {}  ENTER save  ESC cancel",name)}else if searching{format!("SEARCH: {}  ENTER/Esc finish",search)}else{format!("{}  |  / Search  |  f native folder  |  Q Quit",message)};
                f.render_widget(Paragraph::new(footer).style(Style::default().fg(Color::Yellow)).block(Block::default().borders(Borders::ALL)),areas[2]);
            })?;
            if let Event::Key(k)=event::read()? {
                if let Some(path)=pending_delete.take(){match k.code{KeyCode::Char('y')|KeyCode::Char('Y')=>{let metadata=fs::symlink_metadata(&path)?;if metadata.file_type().is_symlink() || metadata.is_file(){fs::remove_file(&path)?;}else if metadata.is_dir(){fs::remove_dir_all(&path)?;}marked.retain(|p|p!=&path);message="Deleted item".into();},_=>message="Delete cancelled".into()}continue;}
                if let Some(buffer)=rename.as_mut(){match k.code{KeyCode::Esc=>{rename=None;message="Rename cancelled".into();},KeyCode::Backspace=>{buffer.pop();},KeyCode::Enter=>{if let Some(old)=entries.get(selected){let name=buffer.trim();if name.is_empty()||name.contains('/')||name.contains('\\'){message="Invalid filename".into();}else{let dest=current.join(name);if dest.exists(){message="That name already exists".into();}else{fs::rename(old,dest)?;message="Renamed item".into();}}}rename=None;},KeyCode::Char(c)if !c.is_control()=>buffer.push(c),_=>{}}continue;}
                if searching{match k.code{KeyCode::Esc|KeyCode::Enter=>searching=false,KeyCode::Backspace=>{search.pop();selected=0;},KeyCode::Char(c)if !c.is_control()=>{search.push(c);selected=0;},_=>{}}continue;}
                if k.modifiers.contains(KeyModifiers::CONTROL)&&matches!(k.code,KeyCode::Char('v')|KeyCode::Char('V')){let mut count=0;for source in &clipboard{if source.is_dir() && current.starts_with(source){message=format!("Cannot paste {} into itself or a child directory",source.display());continue;}let name=source.file_name().unwrap_or_default();let mut dest=current.join(name);if dest.exists(){let stem=source.file_stem().unwrap_or_default().to_string_lossy();let ext=source.extension().and_then(|e|e.to_str()).map(|e|format!(".{e}")).unwrap_or_default();let mut n=1;loop{dest=current.join(format!("{stem}-copy-{n}{ext}"));if !dest.exists(){break;}n+=1;}}copy_item(source,&dest)?;count+=1;}message=format!("Pasted {count} item(s)");continue;}
                match k.code{
                    KeyCode::Esc|KeyCode::Char('q')=>break,
                    KeyCode::Up=>selected=selected.saturating_sub(1),
                    KeyCode::Down=>if selected+1<entries.len(){selected+=1;},
                    KeyCode::PageUp=>selected=selected.saturating_sub(10),
                    KeyCode::PageDown=>selected=(selected+10).min(entries.len().saturating_sub(1)),
                    KeyCode::Enter=>if let Some(path)=entries.get(selected){if path.is_dir(){current=path.clone();selected=0;search.clear();}else{platform::open_file(path)?;message="Opened file in default application".into();}},
                    KeyCode::Backspace=>if let Some(parent)=current.parent(){current=parent.to_path_buf();selected=0;search.clear();},
                    KeyCode::Char('/')=>{search.clear();searching=true;},
                    KeyCode::Char(' ')=>if let Some(path)=entries.get(selected){if marked.contains(path){marked.retain(|p|p!=path);}else{marked.push(path.clone());}},
                    KeyCode::Char('c')=>{clipboard=if marked.is_empty(){entries.get(selected).cloned().into_iter().collect()}else{marked.clone()};message=format!("Copied {} item(s) to CyberShot file clipboard",clipboard.len());},
                    KeyCode::Char('r')=>if let Some(path)=entries.get(selected){rename=Some(path.file_name().unwrap_or_default().to_string_lossy().into_owned());},
                    KeyCode::Char('d')=>if let Some(path)=entries.get(selected){pending_delete=Some(path.clone());},
                    KeyCode::Char('f')=>{platform::open_file_manager(&current)?;message="Opened native file manager".into();},
                    _=>{}
                }
            }
        }
        Ok(())
    })();
    disable_raw_mode()?;execute!(term.backend_mut(),LeaveAlternateScreen)?;term.show_cursor()?;result
}

fn copy_item(source:&std::path::Path,destination:&std::path::Path)->Result<()>{
    if fs::symlink_metadata(source)?.file_type().is_symlink() { anyhow::bail!("Copying symbolic links is not supported by the built-in file clipboard"); }
    if source.is_dir(){fs::create_dir(destination)?;for entry in fs::read_dir(source)?{let entry=entry?;copy_item(&entry.path(),&destination.join(entry.file_name()))?;}}else{fs::copy(source,destination)?;}Ok(())
}

/// Browse, open, rename, delete, and copy captured screenshots.
pub fn history_tui() -> Result<()> {
    let mut term = terminal()?;
    let result = (|| -> Result<()> {
        let dir = config::screenshot_dir()?;
        let mut selected = 0usize;
        let mut msg = String::from("↑/↓ Select  ENTER View  C Copy  R Rename  D Delete  ESC Quit");
        let mut rename: Option<String> = None;

        loop {
            let entries = read_images(&dir)?;
            selected = if entries.is_empty() { 0 } else { selected.min(entries.len() - 1) };
            term.draw(|f| {
                let chunks = Layout::default().direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
                    .split(f.area());
                f.render_widget(
                    Paragraph::new(format!(" CYBERSHOT // CAPTURE HISTORY  ·  {} items ", entries.len()))
                        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                        .block(Block::default().borders(Borders::ALL)), chunks[0]);
                let items: Vec<ListItem> = entries.iter().map(|p| {
                    let name = p.file_name().unwrap_or_default().to_string_lossy();
                    let size = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                    ListItem::new(format!("  ◈  {name}   ({:.1} KB)", size as f64 / 1024.0))
                }).collect();
                let mut state = ListState::default();
                if !items.is_empty() { state.select(Some(selected)); }
                f.render_stateful_widget(List::new(items)
                    .block(Block::default().title(" RECENT CAPTURES ").borders(Borders::ALL))
                    .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .highlight_symbol("  > "), chunks[1], &mut state);
                let footer = if let Some(value) = &rename {
                    format!("Rename to: {value}  ·  ENTER confirm  ·  ESC cancel")
                } else { msg.clone() };
                f.render_widget(Paragraph::new(footer).style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL)), chunks[2]);
            })?;

            if let Event::Key(k) = event::read()? {
                if let Some(buffer) = rename.as_mut() {
                    match k.code {
                        KeyCode::Esc => { rename = None; msg = "Rename cancelled".into(); }
                        KeyCode::Enter => {
                            if let Some(old) = entries.get(selected) {
                                let clean = buffer.trim();
                                if clean.is_empty() { msg = "Filename cannot be empty".into(); }
                                else {
                                    let mut filename = clean.to_string();
                                    if std::path::Path::new(&filename).extension().is_none() {
                                        filename.push_str(old.extension().and_then(|e| e.to_str()).map(|e| format!(".{e}")).unwrap_or_default().as_str());
                                    }
                                    let destination = dir.join(filename);
                                    if destination.exists() { msg = "A file with that name already exists".into(); }
                                    else { fs::rename(old, destination)?; msg = "Screenshot renamed".into(); }
                                }
                            }
                            rename = None;
                        }
                        KeyCode::Backspace => { buffer.pop(); }
                        KeyCode::Char(c) if !c.is_control() && c != '/' && c != '\\' => buffer.push(c),
                        _ => {}
                    }
                    continue;
                }
                match k.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => selected = selected.saturating_add(1),
                    KeyCode::Enter | KeyCode::Char('o') => if let Some(path) = entries.get(selected) {
                        platform::open_file(path)?; msg = format!("Opened {}", path.file_name().unwrap_or_default().to_string_lossy());
                    },
                    KeyCode::Char('c') => if let Some(path) = entries.get(selected) {
                        let image = image::load_from_memory(&fs::read(path)?)?;
                        let mut png = std::io::Cursor::new(Vec::new());
                        image.write_to(&mut png, image::ImageFormat::Png)?;
                        crate::clipboard::set_image(png.get_ref())?;
                        msg = "Screenshot copied to clipboard".into();
                    },
                    KeyCode::Char('r') => if let Some(path) = entries.get(selected) {
                        rename = Some(path.file_stem().unwrap_or_default().to_string_lossy().into_owned());
                    },
                    KeyCode::Char('d') => if let Some(path) = entries.get(selected) {
                        fs::remove_file(path)?; selected = selected.saturating_sub(1); msg = "Screenshot deleted".into();
                    },
                    KeyCode::Char('f') => { platform::open_file_manager(&dir)?; msg = "Opened screenshot folder".into(); },
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

fn read_images(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)?.filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()).is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp")))
        .collect::<Vec<_>>();
    files.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
    files.reverse();
    Ok(files)
}
