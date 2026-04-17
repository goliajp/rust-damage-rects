//! Visual demo for `damage-rects`.
//!
//! Click to add dirty rectangles, press `Space` to "render" the merged
//! damage region (drawn as a green outline), `F` to toggle the
//! full-damage flag, `C` to clear.
//!
//! Run: `cargo run --example visualize -p damage-rects`.

use damage_rects::{DamageRect, DamageTracker};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const WIN_W: u32 = 800;
const WIN_H: u32 = 600;
const ADD_W: f32 = 80.0;
const ADD_H: f32 = 60.0;

struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    tracker: DamageTracker,
    cursor: (f64, f64),
    // Last merged region to visualise briefly after a "render" press.
    last_merged: Option<DamageRect>,
    last_merged_was_full: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            surface: None,
            tracker: DamageTracker::new(),
            cursor: (0.0, 0.0),
            last_merged: None,
            last_merged_was_full: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("damage-rects — click to add | Space: render | F: full | C: clear")
                    .with_inner_size(LogicalSize::new(WIN_W, WIN_H)),
            )
            .expect("create window");
        let window = Rc::new(window);
        let context = Context::new(window.clone()).expect("softbuffer context");
        let surface = Surface::new(&context, window.clone()).expect("softbuffer surface");

        self.window = Some(window);
        self.surface = Some(surface);

        println!("damage-rects visualize");
        println!("  click:  add a dirty rect");
        println!("  Space:  render merged damage (draws green outline + clears)");
        println!("  F:      toggle full-damage flag");
        println!("  C:      clear damage");
        println!("  Q:      quit");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(surface) = &mut self.surface {
                    let Some(w) = NonZeroU32::new(size.width) else { return };
                    let Some(h) = NonZeroU32::new(size.height) else { return };
                    surface.resize(w, h).expect("softbuffer resize");
                }
                self.request_redraw();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
            }

            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let (cx, cy) = self.cursor;
                let rect = DamageRect::new(
                    cx as f32 - ADD_W * 0.5,
                    cy as f32 - ADD_H * 0.5,
                    ADD_W,
                    ADD_H,
                );
                self.tracker.add(rect);
                self.last_merged = None;
                self.request_redraw();
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state: ElementState::Pressed, .. },
                ..
            } => {
                let handled = match &logical_key {
                    Key::Named(NamedKey::Space) => {
                        // "Render" the merged damage — visualise the region, then clear.
                        if self.tracker.is_full() {
                            self.last_merged = None;
                            self.last_merged_was_full = true;
                        } else {
                            self.last_merged = self.tracker.merged();
                            self.last_merged_was_full = false;
                        }
                        self.tracker.clear();
                        true
                    }
                    Key::Character(c) => match c.as_str() {
                        "f" | "F" => {
                            if self.tracker.is_full() {
                                self.tracker.clear();
                            } else {
                                self.tracker.mark_full();
                            }
                            self.last_merged = None;
                            self.last_merged_was_full = false;
                            true
                        }
                        "c" | "C" => {
                            self.tracker.clear();
                            self.last_merged = None;
                            self.last_merged_was_full = false;
                            true
                        }
                        "q" | "Q" => {
                            event_loop.exit();
                            true
                        }
                        _ => false,
                    },
                    _ => false,
                };
                if handled {
                    self.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => self.redraw(),

            _ => {}
        }
    }
}

impl App {
    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn redraw(&mut self) {
        let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) else {
            return;
        };
        let size = window.inner_size();
        let Some(w_nz) = NonZeroU32::new(size.width) else { return };
        let Some(h_nz) = NonZeroU32::new(size.height) else { return };
        surface.resize(w_nz, h_nz).expect("resize");

        let mut buf = surface.buffer_mut().expect("buffer");
        let (w, h) = (size.width as usize, size.height as usize);

        // Background.
        let bg: u32 = if self.tracker.is_full() {
            0xFF_FF_E0_80 // yellow-ish tint — full-damage mode
        } else {
            0xFF_20_20_28
        };
        for p in buf.iter_mut() {
            *p = bg;
        }

        // Dirty rects — semi-opaque red fill + brighter outline.
        for r in self.tracker.rects() {
            fill_rect(&mut buf, w, h, r, 0xFF_C0_40_40);
            draw_outline(&mut buf, w, h, r, 0xFF_FF_80_80);
        }

        // Last-merged overlay (green outline), shown until the next mutation.
        if let Some(m) = self.last_merged {
            draw_outline_thick(&mut buf, w, h, &m, 0xFF_40_E0_60);
        } else if self.last_merged_was_full {
            // Full-damage "render" — outline the whole viewport.
            let full = DamageRect::new(0.0, 0.0, w as f32, h as f32);
            draw_outline_thick(&mut buf, w, h, &full, 0xFF_40_E0_60);
        }

        buf.present().expect("present");

        // Status line printed to stderr (no text rendering in buffer).
        eprint!(
            "\r  rects: {:3}  full: {}  area_upper_bound: {:>8.0}  merged: {:>12}   ",
            self.tracker.len(),
            if self.tracker.is_full() { "YES" } else { " no" },
            self.tracker.area_upper_bound(),
            self.tracker
                .merged()
                .map(|r| format!("{r}"))
                .unwrap_or_else(|| "—".into()),
        );
    }
}

fn fill_rect(buf: &mut [u32], w: usize, h: usize, r: &DamageRect, color: u32) {
    let x0 = r.x.max(0.0) as isize;
    let y0 = r.y.max(0.0) as isize;
    let x1 = (r.right() as isize).min(w as isize);
    let y1 = (r.bottom() as isize).min(h as isize);
    for y in y0..y1 {
        for x in x0..x1 {
            buf[y as usize * w + x as usize] = color;
        }
    }
}

fn draw_outline(buf: &mut [u32], w: usize, h: usize, r: &DamageRect, color: u32) {
    draw_outline_n(buf, w, h, r, color, 1);
}

fn draw_outline_thick(buf: &mut [u32], w: usize, h: usize, r: &DamageRect, color: u32) {
    draw_outline_n(buf, w, h, r, color, 3);
}

fn draw_outline_n(buf: &mut [u32], w: usize, h: usize, r: &DamageRect, color: u32, n: isize) {
    let x0 = r.x as isize;
    let y0 = r.y as isize;
    let x1 = r.right() as isize;
    let y1 = r.bottom() as isize;
    let put = |buf: &mut [u32], x: isize, y: isize| {
        if (0..w as isize).contains(&x) && (0..h as isize).contains(&y) {
            buf[y as usize * w + x as usize] = color;
        }
    };
    for i in 0..n {
        for x in x0..x1 {
            put(buf, x, y0 + i);
            put(buf, x, y1 - 1 - i);
        }
        for y in y0..y1 {
            put(buf, x0 + i, y);
            put(buf, x1 - 1 - i, y);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run app");
}
