//! Window title, geometry, quit and dirty state.

#[allow(unused_imports)]
use super::*;

impl Tk {
    /// Sets the window title (tkinter `root.title`).
    pub fn title(&mut self, title: impl Into<String>) {
        self.window_title = title.into();
        self.dirty = true;
    }

    /// Requests an initial window size in physical px (tkinter `root.geometry("WxH")`).
    pub fn geometry(&mut self, width: u32, height: u32) {
        self.window_size = Some((width, height));
        self.sim_resize(width, height);
    }

    /// Asks the event loop to exit after the current iteration (tkinter `root.quit`).
    pub fn quit(&mut self) {
        self.quit_requested = true;
    }

    /// True once [`Tk::quit`] was called or the root was destroyed.
    pub fn is_quit_requested(&self) -> bool {
        self.quit_requested
    }

    /// True if layout/paint is pending.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

}
