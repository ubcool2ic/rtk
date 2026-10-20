//! Clipboard support: an always-available in-memory buffer plus, when the
//! `clipboard` feature is enabled and the real event loop is running, the OS
//! clipboard via `arboard`. Headless tests never touch the OS clipboard.

#[allow(unused_imports)]
use super::*;

impl Tk {
    /// Lazily creates and returns the OS clipboard handle, if the feature is
    /// enabled and the real event loop is running.
    #[cfg(feature = "clipboard")]
    fn os_clipboard_handle(&mut self) -> Option<&mut arboard::Clipboard> {
        if !self.timers.real_clock {
            return None;
        }
        if self.os_clipboard.is_none() {
            match arboard::Clipboard::new() {
                Ok(clip) => self.os_clipboard = Some(clip),
                Err(_) => return None,
            }
        }
        self.os_clipboard.as_mut()
    }

    /// Clears the clipboard (tkinter `clipboard_clear`).
    pub fn clipboard_clear(&mut self) {
        self.clipboard_text.clear();
        #[cfg(feature = "clipboard")]
        {
            if let Some(clip) = self.os_clipboard_handle() {
                let _ = clip.clear();
            }
        }
    }

    /// Appends `text` to the clipboard (tkinter `clipboard_append`).
    pub fn clipboard_append(&mut self, text: &str) {
        self.clipboard_text.push_str(text);
        #[cfg(feature = "clipboard")]
        {
            let snapshot = self.clipboard_text.clone();
            if let Some(clip) = self.os_clipboard_handle() {
                let _ = clip.set_text(snapshot);
            }
        }
    }

    /// Returns the clipboard contents, if any (tkinter `clipboard_get`).
    ///
    /// When the real event loop is running and the feature is enabled, the OS
    /// clipboard is consulted first; on error the in-memory value is used.
    pub fn clipboard_get(&mut self) -> Option<String> {
        #[cfg(feature = "clipboard")]
        {
            if let Some(clip) = self.os_clipboard_handle() {
                if let Ok(text) = clip.get_text() {
                    self.clipboard_text = text.clone();
                    return Some(text);
                }
            }
        }
        if self.clipboard_text.is_empty() {
            None
        } else {
            Some(self.clipboard_text.clone())
        }
    }
}
