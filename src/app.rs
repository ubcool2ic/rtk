use std::error::Error;
use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use crate::tk::Edit;
use crate::Tk;

struct App {
    tk: Tk,
    // Surface is dropped before context and window.
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    _context: Option<Context<Arc<Window>>>,
    window: Option<Arc<Window>>,
    pointer: (f32, f32),
    modifiers: ModifiersState,
    error: Option<Box<dyn Error>>,
}

impl Tk {
    /// Consumes the toolkit and runs winit. Call this on the main thread.
    pub fn mainloop(self) -> Result<(), Box<dyn Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        let mut app = App {
            tk: self,
            surface: None,
            _context: None,
            window: None,
            pointer: (0.0, 0.0),
            modifiers: ModifiersState::empty(),
            error: None,
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
                .with_inner_size(PhysicalSize::new(width, height)),
        )?);
        let context = Context::new(window.clone())?;
        let surface = Surface::new(&context, window.clone())?;
        self.surface = Some(surface);
        self._context = Some(context);
        self.window = Some(window);
        self.tk.dirty = true;
        Ok(())
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Box<dyn Error>) {
        self.error = Some(error);
        event_loop.exit();
    }

    fn schedule(&self, event_loop: &ActiveEventLoop) {
        if self.tk.quit_requested {
            event_loop.exit();
        } else if self.tk.dirty {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        let Some(window) = &self.window else { return Ok(()) };
        window.set_title(&self.tk.window_title);
        let size = window.inner_size();
        let (Some(width), Some(height)) = (
            NonZeroU32::new(size.width),
            NonZeroU32::new(size.height),
        ) else {
            self.tk.dirty = false;
            return Ok(());
        };
        let pixmap = self.tk.render_to_pixmap(size.width, size.height);
        let Some(surface) = &mut self.surface else { return Ok(()) };
        surface.resize(width, height)?;
        let mut buffer = surface.buffer_mut()?;
        for (pixel, rgba) in buffer.iter_mut().zip(pixmap.data().chunks_exact(4)) {
            // tiny-skia: premultiplied RGBA bytes; softbuffer: integer 0x00RRGGBB.
            let alpha = u32::from(rgba[3]);
            let channel = |value: u8| {
                if alpha == 0 {
                    0
                } else {
                    ((u32::from(value) * 255 + alpha / 2) / alpha).min(255)
                }
            };
            *pixel = channel(rgba[0]) << 16 | channel(rgba[1]) << 8 | channel(rgba[2]);
        }
        buffer.present()?;
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            if let Err(error) = self.open(event_loop) {
                self.fail(event_loop, error);
                return;
            }
        }
        self.schedule(event_loop);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.surface = None;
        self._context = None;
        self.window = None;
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
            WindowEvent::Resized(_) => self.tk.dirty = true,
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = (position.x as f32, position.y as f32);
                self.tk.pointer_move(self.pointer.0, self.pointer.1);
            }
            WindowEvent::CursorLeft { .. } => {
                self.tk.hovered = None;
                self.tk.dirty = true;
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                // Input may arrive between a resize/configuration and its redraw.
                if self.tk.dirty {
                    if let Some(window) = &self.window {
                        let size = window.inner_size();
                        self.tk.layout(size.width, size.height);
                    }
                }
                if state == ElementState::Pressed {
                    self.tk.pointer_press(self.pointer.0, self.pointer.1);
                } else {
                    self.tk.pointer_release(self.pointer.0, self.pointer.1);
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let edit = match &event.logical_key {
                    Key::Named(NamedKey::Backspace) => Some(Edit::Backspace),
                    Key::Named(NamedKey::Delete) => Some(Edit::Delete),
                    Key::Named(NamedKey::ArrowLeft) => Some(Edit::Left),
                    Key::Named(NamedKey::ArrowRight) => Some(Edit::Right),
                    Key::Named(NamedKey::Home) => Some(Edit::Home),
                    Key::Named(NamedKey::End) => Some(Edit::End),
                    _ if !self.modifiers.control_key() && !self.modifiers.super_key() => {
                        event.text.as_ref().map(|text| Edit::Insert(text.to_string()))
                    }
                    _ => None,
                };
                if let Some(edit) = edit {
                    self.tk.edit(edit);
                }
            }
            WindowEvent::Focused(false) => {
                self.tk.focused = None;
                self.tk.pressed = None;
                self.tk.hovered = None;
                self.tk.dirty = true;
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.draw() {
                    self.fail(event_loop, error);
                    return;
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
