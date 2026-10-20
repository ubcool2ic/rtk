//! Window title, geometry, icon, size constraints, close protocol and dirty state.

#[allow(unused_imports)]
use super::*;

impl Tk {
    /// Sets the window title (tkinter `root.title`).
    pub fn title(&mut self, title: impl Into<String>) {
        self.window_title = title.into();
        self.dirty = true;
    }

    /// Returns the current window title.
    pub fn window_title(&self) -> &str {
        &self.window_title
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

    /// Sets the window icon from straight (non-premultiplied) RGBA bytes.
    ///
    /// `rgba` must contain exactly `width * height * 4` bytes and both
    /// dimensions must be non-zero. Returns [`RtkError::InvalidImageSize`]
    /// otherwise. The icon is applied to the real window on the next frame.
    pub fn iconphoto_rgba(
        &mut self,
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<(), RtkError> {
        if width == 0 || height == 0 {
            return Err(RtkError::InvalidImageSize);
        }
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|px| px.checked_mul(4))
            .ok_or(RtkError::InvalidImageSize)?;
        if rgba.len() != expected {
            return Err(RtkError::InvalidImageSize);
        }
        self.window_icon = Some((width, height, rgba));
        self.window_props_dirty = true;
        self.dirty = true;
        Ok(())
    }

    /// Sets the window icon from PNG bytes (tkinter `iconphoto`).
    ///
    /// The PNG is decoded with `tiny-skia`; premultiplied pixels are converted
    /// back to straight RGBA before being stored. Returns
    /// [`RtkError::InvalidImageSize`] if the PNG cannot be decoded or is empty.
    pub fn iconphoto_png(&mut self, png: &[u8]) -> Result<(), RtkError> {
        let pixmap = tiny_skia::Pixmap::decode_png(png).map_err(|_| RtkError::InvalidImageSize)?;
        let width = pixmap.width();
        let height = pixmap.height();
        if width == 0 || height == 0 {
            return Err(RtkError::InvalidImageSize);
        }
        let mut rgba = Vec::with_capacity((width as usize) * (height as usize) * 4);
        for pixel in pixmap.pixels() {
            let c = pixel.demultiply();
            rgba.push(c.red());
            rgba.push(c.green());
            rgba.push(c.blue());
            rgba.push(c.alpha());
        }
        self.iconphoto_rgba(rgba, width, height)
    }

    /// Returns the size of the current window icon, if any.
    pub fn window_icon(&self) -> Option<(u32, u32)> {
        self.window_icon.as_ref().map(|(w, h, _)| (*w, *h))
    }

    /// Returns the raw straight-RGBA bytes of the current window icon, if any.
    pub(crate) fn window_icon_rgba(&self) -> Option<(u32, u32, &[u8])> {
        self.window_icon
            .as_ref()
            .map(|(w, h, data)| (*w, *h, data.as_slice()))
    }

    /// Sets the minimum window size in logical px (tkinter `minsize`).
    pub fn minsize(&mut self, width: u32, height: u32) {
        self.window_minsize = Some((width, height));
        self.window_props_dirty = true;
        self.dirty = true;
    }

    /// Clears the minimum window size constraint (tkinter `minsize` reset).
    pub fn clear_minsize(&mut self) {
        self.window_minsize = None;
        self.window_props_dirty = true;
        self.dirty = true;
    }

    /// Sets the maximum window size in logical px (tkinter `maxsize`).
    pub fn maxsize(&mut self, width: u32, height: u32) {
        self.window_maxsize = Some((width, height));
        self.window_props_dirty = true;
        self.dirty = true;
    }

    /// Clears the maximum window size constraint (tkinter `maxsize` reset).
    pub fn clear_maxsize(&mut self) {
        self.window_maxsize = None;
        self.window_props_dirty = true;
        self.dirty = true;
    }

    /// Sets whether the window can be resized horizontally and/or vertically.
    ///
    /// winit exposes a single resizable flag, so `resizable = width || height`.
    /// When only one axis is resizable, the other is pinned by setting both
    /// min and max size on that axis to the current window size.
    pub fn resizable(&mut self, width: bool, height: bool) {
        self.window_resizable = (width, height);
        self.window_props_dirty = true;
        self.dirty = true;
    }

    /// Returns the minimum window size, if set.
    pub fn window_minsize(&self) -> Option<(u32, u32)> {
        self.window_minsize
    }

    /// Returns the maximum window size, if set.
    pub fn window_maxsize(&self) -> Option<(u32, u32)> {
        self.window_maxsize
    }

    /// Returns whether the window is resizable horizontally and vertically.
    pub fn is_resizable(&self) -> (bool, bool) {
        self.window_resizable
    }

    /// Registers a callback for the window close button (tkinter
    /// `protocol("WM_DELETE_WINDOW", ...)`).
    ///
    /// The callback receives the toolkit and returns `true` to allow the
    /// window to close (quit) or `false` to veto the close.
    pub fn protocol_close(&mut self, callback: impl FnMut(&mut Tk) -> bool + 'static) {
        self.close_callback = Some(Box::new(callback));
    }

    /// Simulates a window close request headlessly.
    ///
    /// Runs the registered [`Tk::protocol_close`] callback (if any) and
    /// returns whether the application will quit as a result.
    pub fn sim_close_request(&mut self) -> bool {
        self.handle_close_request()
    }

    /// Runs the close-request path, returning whether the app will quit.
    ///
    /// The callback is taken out of the toolkit before being invoked so it can
    /// safely re-enter the toolkit; it is restored afterwards unless the
    /// callback itself replaced it.
    pub(crate) fn handle_close_request(&mut self) -> bool {
        let mut callback = self.close_callback.take();
        let allow = match callback.as_mut() {
            Some(cb) => cb(self),
            None => true,
        };
        if self.close_callback.is_none() {
            self.close_callback = callback;
        }
        if allow {
            self.quit();
        }
        allow
    }

    /// Returns true if window properties (icon, size constraints, resizable)
    /// changed since the last call, clearing the flag.
    pub(crate) fn take_window_props_dirty(&mut self) -> bool {
        let dirty = self.window_props_dirty;
        self.window_props_dirty = false;
        dirty
    }
}
