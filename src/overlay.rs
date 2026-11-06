//! Frozen-frame region selector. The screenshot is taken before the GTK window exists.
use crate::{clipboard, config};
use anyhow::{Context, Result};
use gtk::prelude::*;
use gtk4 as gtk;
use image::{GenericImageView, ImageFormat};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

#[derive(Default, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
struct State {
    start: Option<(f64, f64)>,
    rect: Rect,
    finished: bool,
    cancelled: bool,
}

pub fn run() -> Result<()> {
    let dir = config::screenshot_dir()?;
    let snapshot = dir.join(format!(".cybershot-frame-{}.png", std::process::id()));
    // Take the frame before showing any overlay UI.
    let status = std::process::Command::new("grim")
        .arg(&snapshot)
        .status()
        .context("Install grim to use the selection overlay")?;
    if !status.success() {
        anyhow::bail!("Could not capture frozen desktop frame");
    }
    let app = gtk::Application::builder()
        .application_id("io.cybershot.selector")
        .build();
    let snap = snapshot.clone();
    app.connect_activate(move |app| {
        if let Err(e) = show(app, &snap) {
            eprintln!("CyberShot overlay: {e:#}");
        }
    });
    // Do not pass the process arguments (for example, the `hotkey` subcommand)
    // to GApplication. GTK otherwise interprets them as files to open and emits
    // "This application can not open files" instead of activating the selector.
    // Supply only argv[0], as this application does not accept file arguments.
    app.run_with_args(&["cybershot-selector"]);
    let _ = std::fs::remove_file(&snapshot);
    Ok(())
}

fn show(app: &gtk::Application, snapshot: &PathBuf) -> Result<()> {
    let dir = config::screenshot_dir()?;
    let img = image::open(snapshot)?;
    let (iw, ih) = img.dimensions();
    let mut png_file = std::fs::File::open(snapshot)?;
    let surface = gtk::cairo::ImageSurface::create_from_png(&mut png_file)
        .context("Cannot load frozen frame")?;
    let state = Rc::new(RefCell::new(State {
        start: None,
        rect: Rect::default(),
        finished: false,
        cancelled: false,
    }));
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("CyberShot Select")
        .decorated(false)
        .build();
    window.fullscreen();
    let area = gtk::DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    let st = state.clone();
    let sf = surface.clone();
    area.set_draw_func(move |_, cr, w, h| {
        let sx = w as f64 / iw as f64;
        let sy = h as f64 / ih as f64;
        cr.scale(sx, sy);
        let _ = cr.set_source_surface(&sf, 0.0, 0.0);
        let _ = cr.paint();
        cr.scale(1.0 / sx, 1.0 / sy);
        let s = st.borrow();
        let r = s.rect;
        if r.w > 1.0 && r.h > 1.0 {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.55);
            cr.rectangle(0.0, 0.0, w as f64, h as f64);
            let _ = cr.fill();
            let _ = cr.set_source_surface(&sf, 0.0, 0.0);
            let _ = cr.paint();
            cr.set_source_rgba(0.15, 0.75, 1.0, 0.95);
            cr.set_line_width(2.0);
            cr.rectangle(r.x, r.y, r.w, r.h);
            let _ = cr.stroke();
        }
    });
    let drag = gtk::GestureDrag::new();
    let st = state.clone();
    let ar = area.clone();
    drag.connect_drag_begin(move |_, x, y| {
        let mut s = st.borrow_mut();
        s.start = Some((x, y));
        s.rect = Rect {
            x,
            y,
            w: 0.0,
            h: 0.0,
        };
        ar.queue_draw();
    });
    let st = state.clone();
    let ar = area.clone();
    drag.connect_drag_update(move |_, dx, dy| {
        let mut s = st.borrow_mut();
        if let Some((x, y)) = s.start {
            let x2 = x + dx;
            let y2 = y + dy;
            s.rect = Rect {
                x: x.min(x2),
                y: y.min(y2),
                w: (x2 - x).abs(),
                h: (y2 - y).abs(),
            };
        }
        ar.queue_draw();
    });
    area.add_controller(drag);
    let keys = gtk::EventControllerKey::new();
    let st = state.clone();
    let win = window.clone();
    keys.connect_key_pressed(move |_, key, _, mods| {
        use gtk::gdk::Key;
        let mut s = st.borrow_mut();
        match key {
            Key::Escape => {
                s.cancelled = true;
                win.close();
                glib::Propagation::Stop
            }
            Key::r | Key::R => {
                s.rect = Rect::default();
                s.start = None;
                drop(s);
                win.queue_draw();
                glib::Propagation::Stop
            }
            Key::s | Key::S if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK) => {
                if s.rect.w > 3.0 && s.rect.h > 3.0 {
                    s.finished = true;
                    win.close();
                }
                glib::Propagation::Stop
            }
            Key::Return | Key::KP_Enter => {
                if s.rect.w > 3.0 && s.rect.h > 3.0 {
                    s.finished = true;
                    win.close();
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    window.add_controller(keys);
    window.set_child(Some(&area));
    window.present();
    // Keep GTK's main loop running until the user confirms or cancels.
    let loop_ = glib::MainLoop::new(None, false);
    let lp = loop_.clone();
    window.connect_close_request(move |_| {
        lp.quit();
        glib::Propagation::Proceed
    });
    loop_.run();
    let result = {
        let s = state.borrow();
        if s.cancelled {
            None
        } else if s.finished {
            Some(s.rect)
        } else {
            None
        }
    };
    if let Some(r) = result {
        let alloc = area.allocation();
        let sx = iw as f64 / alloc.width().max(1) as f64;
        let sy = ih as f64 / alloc.height().max(1) as f64;
        let x = (r.x * sx).round().max(0.0) as u32;
        let y = (r.y * sy).round().max(0.0) as u32;
        let w = (r.w * sx).round().max(1.0) as u32;
        let h = (r.h * sy).round().max(1.0) as u32;
        let crop = img.crop_imm(
            x,
            y,
            w.min(iw.saturating_sub(x)),
            h.min(ih.saturating_sub(y)),
        );
        let path = dir.join(format!(
            "{}-selection.png",
            chrono::Local::now().format("%Y%m%d-%H%M%S-%3f")
        ));
        crop.save_with_format(&path, ImageFormat::Png)?;
        clipboard::set_image(&std::fs::read(&path)?)?;
        println!("Saved and copied: {}", path.display());
    }
    Ok(())
}
