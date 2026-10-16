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
}

impl Tk {
    /// Consumes the toolkit and runs winit. Call on the main thread.
    pub fn mainloop(self) -> Result<(), Box<dyn Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        let now = Instant::now();
        let mut app = App {
            tk: self,
            surface: None,
            context: None,
            window: None,
            modifiers: ModifiersState::empty(),
            occluded: false,
            error: None,
            last_frame: now,
            last_sheet_poll: now,
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

        // Hot reload check every 500ms
        let now = Instant::now();
        if now.duration_since(self.last_sheet_poll) >= Duration::from_millis(500) {
            self.last_sheet_poll = now;
            self.tk.poll_stylesheet();
        }

        if self.tk.is_animating() {
            let elapsed = now.duration_since(self.last_frame).as_millis() as u32;
            if elapsed > 0 {
                self.tk.sim_advance_time(elapsed);
                self.last_frame = now;
            }
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(16)));
        } else if self.tk.dirty && !self.occluded {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(500)));
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(500)));
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
        let pixmap = self.tk.render_to_pixmap(size.width, size.height)?;
        let Some(surface) = &mut self.surface else { return Ok(()) };
        surface.resize(width, height)?;
        let mut buffer = surface.buffer_mut()?;
        for (pixel, rgba) in buffer.iter_mut().zip(pixmap.data().chunks_exact(4)) {
            let [r, g, b, a] = rgba else { continue };
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
        Ok(())
    }
}

impl ApplicationHandler for App {
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
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y / 20.0) as f32,
                };
                self.tk.sim_wheel(dy);
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
                    WinitKey::Named(NamedKey::Tab) => Some(if self.modifiers.shift_key() {
                        Key::ShiftTab
                    } else {
                        Key::Tab
                    }),
                    WinitKey::Named(NamedKey::Enter) => Some(Key::Enter),
                    WinitKey::Named(NamedKey::Space) => Some(Key::Space),
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
        self.schedule(event_loop);
    }
}

pub(crate) fn accepts_key_input(ctrl: bool, alt: bool, logo: bool) -> bool {
    !logo && (!ctrl || alt)
}

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
