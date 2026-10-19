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
use winit::window::{Window, WindowId};

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
        let window = Arc::new(event_loop.create_window(
            Window::default_attributes()
                .with_title(&self.tk.window_title)
                .with_inner_size(LogicalSize::new(width, height)),
        )?);
        self.tk.sim_scale_factor(window.scale_factor());
        let size = window.inner_size();
        self.tk.sim_resize(size.width, size.height);
        self.window = Some(window);
        self.occluded = false;
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
                if alpha == 0 {
                    0
                } else {
                    ((u32::from(value) * 255 + alpha / 2) / alpha).min(255)
                }
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
            WindowEvent::CloseRequested => self.tk.quit(),
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
                let key = match &event.logical_key {
                    WinitKey::Named(NamedKey::Backspace) => Some(Key::Backspace),
                    WinitKey::Named(NamedKey::Delete) => Some(Key::Delete),
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
                    WinitKey::Character(text) if text.as_str() == " " => Some(Key::Space),
                    _ => None,
                };
                if accepts_key_input(
                    self.modifiers.control_key(),
                    self.modifiers.alt_key(),
                    self.modifiers.super_key(),
                ) {
                    if let Some(key) = key {
                        self.tk.sim_key(key);
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

/// Minimum time between presented frames (about 60 Hz).
const FRAME_INTERVAL: Duration = Duration::from_millis(16);

/// Time of the first `Tk::new`, for the opt-in first-frame diagnostic.
pub(crate) static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[cfg(test)]
mod tests {
    use super::accepts_key_input;

    #[test]
    fn altgr_text_is_accepted_but_shortcuts_are_not() {
        assert!(accepts_key_input(false, false, false));
        assert!(accepts_key_input(true, true, false));
        assert!(!accepts_key_input(true, false, false));
        assert!(!accepts_key_input(false, false, true));
    }
}
