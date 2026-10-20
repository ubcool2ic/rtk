use std::error::Error;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey};
use winit::window::{Icon, Window, WindowId};

use crate::shortcut::Modifiers;
use crate::tk::RtkUserEvent;
use crate::{Key, Tk};

struct App {
    tk: Tk,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    context: Option<Context<Arc<Window>>>,
    window: Option<Arc<Window>>,
    modifiers: ModifiersState,
    occluded: bool,
    error: Option<Box<dyn Error>>,
    last_frame: Instant,
    last_sheet_poll: Instant,
    first_frame_done: bool,
    last_draw: Instant,
}

impl Tk {
    /// Consumes the toolkit and runs winit. Call on the main thread.
    pub fn mainloop(self) -> Result<(), Box<dyn Error>> {
        let event_loop = EventLoop::<RtkUserEvent>::with_user_event().build()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        let now = Instant::now();
        let proxy = event_loop.create_proxy();
        if let Ok(mut w) = self.proxy_shared.waker.lock() {
            *w = Some(proxy);
        }
        let mut tk = self;
        tk.timers.real_clock = true;
        let mut app = App {
            tk,
            surface: None,
            context: None,
            window: None,
            modifiers: ModifiersState::empty(),
            occluded: false,
            error: None,
            last_frame: now,
            last_sheet_poll: now,
            first_frame_done: false,
            last_draw: now,
        };
        event_loop.run_app(&mut app)?;
        match app.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl App {
    fn open(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        let (width, height) = self.tk.initial_size();
        let mut attributes = Window::default_attributes()
            .with_title(&self.tk.window_title)
            .with_inner_size(LogicalSize::new(width, height));
        if let Some(icon) = build_icon(&self.tk) {
            attributes = attributes.with_window_icon(Some(icon));
        }
        if let Some((w, h)) = self.tk.window_minsize() {
            attributes = attributes.with_min_inner_size(LogicalSize::new(f64::from(w), f64::from(h)));
        }
        if let Some((w, h)) = self.tk.window_maxsize() {
            attributes = attributes.with_max_inner_size(LogicalSize::new(f64::from(w), f64::from(h)));
        }
        let (rw, rh) = self.tk.is_resizable();
        attributes = attributes.with_resizable(rw || rh);
        let window = Arc::new(event_loop.create_window(attributes)?);
        self.tk.sim_scale_factor(window.scale_factor());
        let size = window.inner_size();
        self.tk.sim_resize(size.width, size.height);
        self.window = Some(window);
        self.occluded = false;
        // Apply size constraints (including single-axis pinning) immediately so
        // the very first frame already respects them.
        self.sync_window_props();
        Ok(())
    }

    fn schedule(&mut self, event_loop: &ActiveEventLoop) {
        if self.tk.quit_requested {
            event_loop.exit();
            return;
        }

        // Drain any thread-safe tasks posted via TkProxy or after_idle.
        if self.tk.process_posted() > 0 && self.tk.quit_requested {
            event_loop.exit();
            return;
        }

        // Fire due timers (after/every) against the real clock.
        let now = Instant::now();
        let start = *START.get_or_init(Instant::now);
        let now_ms = now.duration_since(start).as_millis() as u64;
        if self.tk.run_timers_until(now_ms) > 0 && self.tk.quit_requested {
            event_loop.exit();
            return;
        }

        // Hot reload check every 500ms
        if now.duration_since(self.last_sheet_poll) >= Duration::from_millis(500) {
            self.last_sheet_poll = now;
            self.tk.poll_stylesheet();
        }

        // Cap redraws at ~60 Hz: many input events can arrive per frame.
        let next_frame = self.last_draw + FRAME_INTERVAL;
        if self.tk.is_animating() {
            let elapsed = now.duration_since(self.last_frame).as_millis().min(1000) as u32;
            if elapsed > 0 {
                self.tk.sim_advance_time(elapsed);
                self.last_frame = now;
            }
            if now >= next_frame {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(now + FRAME_INTERVAL));
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame));
            }
        } else if self.tk.dirty && !self.occluded {
            // Animations measure time from when they start, not from the last one.
            self.last_frame = now;
            if now >= next_frame {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                event_loop.set_control_flow(ControlFlow::Wait);
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame));
            }
        } else if self.tk.watched_stylesheet.is_some() {
            self.last_frame = now;
            // Only wake periodically when a stylesheet is being watched for hot reload.
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(500)));
        } else {
            self.last_frame = now;
            // Truly idle: sleep until the OS delivers an event.
            event_loop.set_control_flow(ControlFlow::Wait);
        }

        // Sleep no later than the next timer deadline (no busy-waiting).
        if let Some(due) = self.tk.next_timer_due_ms() {
            let deadline = start + Duration::from_millis(due);
            let wake = match event_loop.control_flow() {
                ControlFlow::WaitUntil(t) => t.min(deadline),
                _ => deadline,
            };
            event_loop.set_control_flow(ControlFlow::WaitUntil(wake));
        }
    }

    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        let Some(window) = self.window.clone() else { return Ok(()) };
        window.set_title(&self.tk.window_title);
        self.apply_window_props();
        let size = window.inner_size();
        if self.occluded || size.width == 0 || size.height == 0 {
            self.tk.dirty = false;
            return Ok(());
        }
        let (Some(width), Some(height)) = (
            NonZeroU32::new(size.width),
            NonZeroU32::new(size.height),
        ) else {
            return Ok(());
        };
        if self.context.is_none() {
            self.context = Some(Context::new(window.clone())?);
        }
        if self.surface.is_none() {
            if let Some(context) = &self.context {
                self.surface = Some(Surface::new(context, window)?);
            }
        }
        self.last_draw = Instant::now();
        let t_render = Instant::now();
        let pixmap = self.tk.render_to_pixmap(size.width, size.height)?;
        let render_time = t_render.elapsed();
        let t_present = Instant::now();
        let Some(surface) = &mut self.surface else { return Ok(()) };
        surface.resize(width, height)?;
        let mut buffer = surface.buffer_mut()?;
        for (pixel, rgba) in buffer.iter_mut().zip(pixmap.data().chunks_exact(4)) {
            let [r, g, b, a] = rgba else { continue };
            if *a == 255 {
                // Opaque (the common case): premultiplied equals straight colour.
                *pixel = (u32::from(*r) << 16) | (u32::from(*g) << 8) | u32::from(*b);
                continue;
            }
            let alpha = u32::from(*a);
            let channel = |value: u8| {
                (u32::from(value) * 255 + alpha / 2)
                    .checked_div(alpha)
                    .map_or(0, |v| v.min(255))
            };
            *pixel = (channel(*r) << 16) | (channel(*g) << 8) | channel(*b);
        }
        buffer.present()?;
        if std::env::var_os("RTK_STATS").is_some() {
            eprintln!(
                "rtk: frame render {:.2} ms, convert+present {:.2} ms",
                render_time.as_secs_f64() * 1000.0,
                t_present.elapsed().as_secs_f64() * 1000.0
            );
        }
        if !self.first_frame_done {
            self.first_frame_done = true;
            // Opt-in diagnostics used by docs/PERFORMANCE.md.
            if std::env::var_os("RTK_FIRST_FRAME").is_some() {
                let ms = START.get().map_or(0.0, |start| start.elapsed().as_secs_f64() * 1000.0);
                eprintln!("rtk: first frame presented {ms:.1} ms after Tk::new");
            }
            if std::env::var_os("RTK_EXIT_AFTER_FIRST_FRAME").is_some() {
                self.tk.quit();
            }
        }
        Ok(())
    }
}

impl ApplicationHandler<RtkUserEvent> for App {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: RtkUserEvent) {
        self.tk.process_posted();
        self.schedule(event_loop);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.tk.quit_requested {
            event_loop.exit();
            return;
        }
        if self.window.is_none() {
            if let Err(error) = self.open(event_loop) {
                eprintln!("rtk: cannot create window: {error}");
                self.error = Some(error);
                event_loop.exit();
                return;
            }
        }
        self.schedule(event_loop);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.surface = None;
        self.context = None;
        self.window = None;
        self.tk.clear_input();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(|window| window.id()) != Some(window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.tk.handle_close_request();
            }
            WindowEvent::Resized(size) => self.tk.sim_resize(size.width, size.height),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.tk.sim_scale_factor(scale_factor);
                if let Some(window) = &self.window {
                    let size = window.inner_size();
                    self.tk.sim_resize(size.width, size.height);
                }
            }
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                self.tk.dirty = !occluded;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = f64::from(self.tk.scale_factor);
                self.tk.sim_mouse_move((position.x / scale) as f32, (position.y / scale) as f32);
            }
            WindowEvent::CursorLeft { .. } => {
                self.tk.sim_mouse_move(f32::NEG_INFINITY, f32::NEG_INFINITY);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    // Shift+wheel scrolls horizontally, like most toolkits.
                    MouseScrollDelta::LineDelta(x, y) if self.modifiers.shift_key() && x == 0.0 => self.tk.sim_wheel_xy(y, 0.0, false),
                    MouseScrollDelta::LineDelta(x, y) => self.tk.sim_wheel_xy(x, y, false),
                    MouseScrollDelta::PixelDelta(pos) => {
                        let scale = f64::from(self.tk.scale_factor);
                        self.tk.sim_wheel_xy((pos.x / scale) as f32, (pos.y / scale) as f32, true);
                    }
                }
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                if state == ElementState::Pressed {
                    self.tk.sim_mouse_down();
                } else {
                    self.tk.sim_mouse_up();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let mods = Modifiers {
                    ctrl: self.modifiers.control_key(),
                    shift: self.modifiers.shift_key(),
                    alt: self.modifiers.alt_key(),
                    meta: self.modifiers.super_key(),
                };
                let shortcut_modifier = mods.ctrl || mods.alt || mods.meta;
                let key = match &event.logical_key {
                    WinitKey::Named(NamedKey::Backspace) => Some(Key::Backspace),
                    WinitKey::Named(NamedKey::Delete) => Some(Key::Delete),
                    WinitKey::Named(NamedKey::Insert) => Some(Key::Insert),
                    WinitKey::Named(NamedKey::ArrowLeft) => Some(Key::Left),
                    WinitKey::Named(NamedKey::ArrowRight) => Some(Key::Right),
                    WinitKey::Named(NamedKey::ArrowUp) => Some(Key::Up),
                    WinitKey::Named(NamedKey::ArrowDown) => Some(Key::Down),
                    WinitKey::Named(NamedKey::Home) => Some(Key::Home),
                    WinitKey::Named(NamedKey::End) => Some(Key::End),
                    WinitKey::Named(NamedKey::PageUp) => Some(Key::PageUp),
                    WinitKey::Named(NamedKey::PageDown) => Some(Key::PageDown),
                    WinitKey::Named(NamedKey::Tab) => Some(if self.modifiers.shift_key() {
                        Key::ShiftTab
                    } else {
                        Key::Tab
                    }),
                    WinitKey::Named(NamedKey::Enter) => Some(Key::Enter),
                    WinitKey::Named(NamedKey::Space) => Some(Key::Space),
                    WinitKey::Named(NamedKey::Escape) => Some(Key::Escape),
                    WinitKey::Named(NamedKey::F1) => Some(Key::F(1)),
                    WinitKey::Named(NamedKey::F2) => Some(Key::F(2)),
                    WinitKey::Named(NamedKey::F3) => Some(Key::F(3)),
                    WinitKey::Named(NamedKey::F4) => Some(Key::F(4)),
                    WinitKey::Named(NamedKey::F5) => Some(Key::F(5)),
                    WinitKey::Named(NamedKey::F6) => Some(Key::F(6)),
                    WinitKey::Named(NamedKey::F7) => Some(Key::F(7)),
                    WinitKey::Named(NamedKey::F8) => Some(Key::F(8)),
                    WinitKey::Named(NamedKey::F9) => Some(Key::F(9)),
                    WinitKey::Named(NamedKey::F10) => Some(Key::F(10)),
                    WinitKey::Named(NamedKey::F11) => Some(Key::F(11)),
                    WinitKey::Named(NamedKey::F12) => Some(Key::F(12)),
                    WinitKey::Named(NamedKey::F13) => Some(Key::F(13)),
                    WinitKey::Named(NamedKey::F14) => Some(Key::F(14)),
                    WinitKey::Named(NamedKey::F15) => Some(Key::F(15)),
                    WinitKey::Named(NamedKey::F16) => Some(Key::F(16)),
                    WinitKey::Named(NamedKey::F17) => Some(Key::F(17)),
                    WinitKey::Named(NamedKey::F18) => Some(Key::F(18)),
                    WinitKey::Named(NamedKey::F19) => Some(Key::F(19)),
                    WinitKey::Named(NamedKey::F20) => Some(Key::F(20)),
                    WinitKey::Named(NamedKey::F21) => Some(Key::F(21)),
                    WinitKey::Named(NamedKey::F22) => Some(Key::F(22)),
                    WinitKey::Named(NamedKey::F23) => Some(Key::F(23)),
                    WinitKey::Named(NamedKey::F24) => Some(Key::F(24)),
                    WinitKey::Character(text) if text.as_str() == " " => Some(Key::Space),
                    WinitKey::Character(text) if shortcut_modifier => {
                        text.chars().next().map(|c| Key::Char(c.to_ascii_lowercase()))
                    }
                    _ => None,
                };
                if accepts_key_input(
                    self.modifiers.control_key(),
                    self.modifiers.alt_key(),
                    self.modifiers.super_key(),
                ) {
                    if let Some(key) = key {
                        self.tk.sim_key_with(key, mods);
                    } else if let Some(text) = event.text {
                        self.tk.sim_text(&text);
                    }
                }
            }
            WindowEvent::Focused(false) => self.tk.clear_input(),
            WindowEvent::RedrawRequested => {
                self.last_frame = Instant::now();
                if let Err(error) = self.draw() {
                    eprintln!("rtk: render/surface error (will retry on the next event): {error}");
                    self.surface = None;
                    self.tk.dirty = false;
                }
            }
            _ => {}
        }
        self.schedule(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.tk.process_posted();
        self.schedule(event_loop);
    }
}

pub(crate) fn accepts_key_input(ctrl: bool, alt: bool, logo: bool) -> bool {
    !logo && (!ctrl || alt)
}

/// Builds a winit [`Icon`] from the toolkit's stored RGBA icon, if any.
fn build_icon(tk: &Tk) -> Option<Icon> {
    let (w, h, rgba) = tk.window_icon_rgba()?;
    Icon::from_rgba(rgba.to_vec(), w, h).ok()
}

impl App {
    /// Re-applies window properties (icon, size constraints, resizable) to the
    /// real window when the toolkit reports they changed.
    fn apply_window_props(&mut self) {
        if !self.tk.take_window_props_dirty() {
            return;
        }
        self.sync_window_props();
    }

    /// Pushes the toolkit's current window properties (icon, size constraints,
    /// resizable flag) onto the real window, regardless of the dirty flag.
    ///
    /// Used both by [`App::apply_window_props`] (when the toolkit reports a
    /// change) and once right after the window is created in [`App::open`] so
    /// that single-axis pinning takes effect from the first frame.
    fn sync_window_props(&mut self) {
        let Some(window) = self.window.clone() else { return };
        window.set_window_icon(build_icon(&self.tk));
        let scale = f64::from(self.tk.scale_factor);
        let to_logical = |(w, h): (u32, u32)| LogicalSize::new(f64::from(w), f64::from(h));
        let (rw, rh) = self.tk.is_resizable();
        window.set_resizable(rw || rh);
        // Pin the non-resizable axis to the current window size, keeping the
        // other axis free within the user's min/max constraints.
        let size = window.inner_size();
        let cur = (
            (f64::from(size.width) / scale).round() as u32,
            (f64::from(size.height) / scale).round() as u32,
        );
        let (min, max) = pinned_limits(
            (rw, rh),
            cur,
            self.tk.window_minsize(),
            self.tk.window_maxsize(),
        );
        if let Some(min) = min {
            window.set_min_inner_size(Some(to_logical(min)));
        } else {
            window.set_min_inner_size(None::<LogicalSize<f64>>);
        }
        if let Some(max) = max {
            window.set_max_inner_size(Some(to_logical(max)));
        } else {
            window.set_max_inner_size(None::<LogicalSize<f64>>);
        }
    }
}

/// A `(width, height)` pair of logical pixels.
type Size = (u32, u32);

/// The effective `(min, max)` inner-size constraints for a window.
type Limits = (Option<Size>, Option<Size>);

/// Computes the effective min/max inner size for a window given its
/// resizability, current size and the user's requested constraints.
///
/// When both axes are resizable the user's constraints are returned verbatim.
/// When exactly one axis is fixed, that axis is pinned to the current size on
/// both min and max, while the free axis keeps the user's constraints (falling
/// back to `1` for min and [`FREE_AXIS_MAX`] for max). When neither axis is
/// resizable both axes are pinned to the current size.
fn pinned_limits(resizable: (bool, bool), cur: Size, min: Option<Size>, max: Option<Size>) -> Limits {
    let (rw, rh) = resizable;
    if rw && rh {
        return (min, max);
    }
    let (min_w, min_h): Size = min.unwrap_or((1, 1));
    let (max_w, max_h): Size = max.unwrap_or((FREE_AXIS_MAX, FREE_AXIS_MAX));
    let (min_w, max_w) = if rw { (min_w, max_w) } else { (cur.0, cur.0) };
    let (min_h, max_h) = if rh { (min_h, max_h) } else { (cur.1, cur.1) };
    // Guard against inverted ranges (e.g. a user min larger than the pinned
    // current size) so winit never receives min > max.
    let min_w = min_w.min(max_w);
    let min_h = min_h.min(max_h);
    let min: Size = (min_w, min_h);
    let max: Size = (max_w, max_h);
    (Some(min), Some(max))
}

/// Upper bound used for a free axis when the user did not set a maximum.
const FREE_AXIS_MAX: u32 = 100_000;

/// Minimum time between presented frames (about 60 Hz).
const FRAME_INTERVAL: Duration = Duration::from_millis(16);

/// Time of the first `Tk::new`, for the opt-in first-frame diagnostic.
pub(crate) static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[cfg(test)]
mod tests {
    use super::{accepts_key_input, pinned_limits, FREE_AXIS_MAX};

    #[test]
    fn altgr_text_is_accepted_but_shortcuts_are_not() {
        assert!(accepts_key_input(false, false, false));
        assert!(accepts_key_input(true, true, false));
        assert!(!accepts_key_input(true, false, false));
        assert!(!accepts_key_input(false, false, true));
    }

    #[test]
    fn fully_resizable_keeps_user_constraints() {
        let min = Some((100, 200));
        let max = Some((800, 600));
        assert_eq!(
            pinned_limits((true, true), (640, 480), min, max),
            (min, max)
        );
        assert_eq!(pinned_limits((true, true), (640, 480), None, None), (None, None));
    }

    #[test]
    fn fixed_width_pins_only_width() {
        let (min, max) = pinned_limits((false, true), (640, 480), None, None);
        assert_eq!(min, Some((640, 1)));
        assert_eq!(max, Some((640, FREE_AXIS_MAX)));
    }

    #[test]
    fn fixed_height_pins_only_height() {
        let (min, max) = pinned_limits((true, false), (640, 480), None, None);
        assert_eq!(min, Some((1, 480)));
        assert_eq!(max, Some((FREE_AXIS_MAX, 480)));
    }

    #[test]
    fn fixed_width_keeps_user_height_constraints() {
        let (min, max) = pinned_limits((false, true), (640, 480), Some((10, 200)), Some((900, 700)));
        assert_eq!(min, Some((640, 200)));
        assert_eq!(max, Some((640, 700)));
    }

    #[test]
    fn fixed_height_keeps_user_width_constraints() {
        let (min, max) = pinned_limits((true, false), (640, 480), Some((300, 10)), Some((900, 700)));
        assert_eq!(min, Some((300, 480)));
        assert_eq!(max, Some((900, 480)));
    }

    #[test]
    fn fully_fixed_pins_both_axes() {
        let (min, max) = pinned_limits((false, false), (640, 480), None, None);
        assert_eq!(min, Some((640, 480)));
        assert_eq!(max, Some((640, 480)));
    }

    #[test]
    fn inverted_ranges_are_clamped() {
        // User min larger than the pinned current width must not produce min > max.
        let (min, max) = pinned_limits((false, true), (100, 480), Some((500, 10)), None);
        assert_eq!(min, Some((100, 10)));
        assert_eq!(max, Some((100, FREE_AXIS_MAX)));
    }
}
