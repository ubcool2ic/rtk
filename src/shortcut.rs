//! Keyboard modifiers, shortcut parsing, menu labels and binding identifiers.

use std::fmt;

use crate::Key;

/// Modifier keys held during a keyboard event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// The Control key.
    pub ctrl: bool,
    /// The Shift key.
    pub shift: bool,
    /// The Alt or Option key.
    pub alt: bool,
    /// The Command, Super or Windows key.
    pub meta: bool,
}

impl Modifiers {
    /// No modifier keys.
    pub const NONE: Self = Self {
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
    };

    /// The Control modifier.
    pub const CTRL: Self = Self {
        ctrl: true,
        ..Self::NONE
    };

    /// The Shift modifier.
    pub const SHIFT: Self = Self {
        shift: true,
        ..Self::NONE
    };

    /// The Alt or Option modifier.
    pub const ALT: Self = Self {
        alt: true,
        ..Self::NONE
    };

    /// The Command, Super or Windows modifier.
    pub const META: Self = Self {
        meta: true,
        ..Self::NONE
    };

    /// Returns Command on macOS and Control on other platforms.
    pub const fn primary() -> Self {
        if cfg!(target_os = "macos") {
            Self::META
        } else {
            Self::CTRL
        }
    }

    /// Returns these modifiers with Shift enabled.
    pub const fn with_shift(self) -> Self {
        Self {
            shift: true,
            ..self
        }
    }

    /// Returns whether the platform's primary modifier is held.
    pub const fn is_primary(&self) -> bool {
        if cfg!(target_os = "macos") {
            self.meta
        } else {
            self.ctrl
        }
    }
}

/// A key and its required modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    /// Required modifier keys.
    pub mods: Modifiers,
    /// The shortcut's key.
    pub key: Key,
}

impl Shortcut {
    /// Parses an accelerator such as `Ctrl+S`, `Primary+Shift+Z` or `F5`.
    ///
    /// Modifier and named-key spelling is case-insensitive. The plus key may
    /// be written as `Plus` or as a literal trailing plus (`Ctrl++`).
    pub fn parse(s: &str) -> Result<Self, ShortcutError> {
        let mut text = s.trim();
        if text.is_empty() {
            return Err(ShortcutError::Empty);
        }

        let mut mods = Modifiers::NONE;
        // Accept the modifier symbols emitted by macOS menu labels as well.
        while let Some(ch) = text.chars().next() {
            match ch {
                '⌃' => mods.ctrl = true,
                '⇧' => mods.shift = true,
                '⌥' => mods.alt = true,
                '⌘' => mods.meta = true,
                _ => break,
            }
            text = &text[ch.len_utf8()..];
        }

        let mut parts: Vec<_> = text.split('+').map(str::trim).collect();
        let mut key_name = parts.pop().unwrap_or_default();

        // A literal plus contributes two trailing empty split components:
        // "Ctrl++" -> ["Ctrl", "", ""], "+" -> ["", ""].
        // A single trailing separator ("Ctrl+") is instead a missing key.
        if key_name.is_empty() && parts.last().copied() == Some("") {
            parts.pop();
            key_name = "+";
        }
        if key_name.is_empty() {
            return Err(ShortcutError::UnknownKey(String::new()));
        }

        for part in parts {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => mods.ctrl = true,
                "shift" => mods.shift = true,
                "alt" | "option" => mods.alt = true,
                "meta" | "cmd" | "command" | "super" | "win" | "windows" => mods.meta = true,
                "primary" => {
                    if cfg!(target_os = "macos") {
                        mods.meta = true;
                    } else {
                        mods.ctrl = true;
                    }
                }
                _ => return Err(ShortcutError::UnknownModifier(part.to_owned())),
            }
        }

        Ok(Self {
            mods,
            key: parse_key(key_name)?,
        })
    }

    /// Formats a menu label using platform-appropriate modifier notation.
    pub fn label(&self) -> String {
        let mut label = String::new();
        if cfg!(target_os = "macos") {
            for (enabled, symbol) in [
                (self.mods.ctrl, "⌃"),
                (self.mods.alt, "⌥"),
                (self.mods.shift, "⇧"),
                (self.mods.meta, "⌘"),
            ] {
                if enabled {
                    label.push_str(symbol);
                }
            }
        } else {
            for (enabled, name) in [
                (self.mods.ctrl, "Ctrl+"),
                (self.mods.shift, "Shift+"),
                (self.mods.alt, "Alt+"),
                (self.mods.meta, "Meta+"),
            ] {
                if enabled {
                    label.push_str(name);
                }
            }
        }
        label.push_str(&key_label(self.key));
        label
    }
}

/// An error encountered while parsing a shortcut.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutError {
    /// The entire shortcut string is empty or whitespace.
    Empty,
    /// The key name is missing or unrecognized.
    UnknownKey(String),
    /// A modifier name is unrecognized.
    UnknownModifier(String),
}

impl fmt::Display for ShortcutError {
    /// Describes the invalid shortcut component.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("shortcut string is empty"),
            Self::UnknownKey(key) => write!(f, "unknown shortcut key: {key}"),
            Self::UnknownModifier(modifier) => {
                write!(f, "unknown shortcut modifier: {modifier}")
            }
        }
    }
}

impl std::error::Error for ShortcutError {}

/// An opaque identifier returned when a shortcut is bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShortcutId(pub(crate) u64);

/// Parses a named key, function key or single character.
fn parse_key(name: &str) -> Result<Key, ShortcutError> {
    let lower = name.to_ascii_lowercase();
    let key = match lower.as_str() {
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "insert" | "ins" => Key::Insert,
        "left" | "arrowleft" => Key::Left,
        "right" | "arrowright" => Key::Right,
        "up" | "arrowup" => Key::Up,
        "down" | "arrowdown" => Key::Down,
        "home" => Key::Home,
        "end" => Key::End,
        "tab" => Key::Tab,
        "shifttab" => Key::ShiftTab,
        "enter" | "return" => Key::Enter,
        "space" | "spacebar" => Key::Space,
        "pageup" | "pgup" => Key::PageUp,
        "pagedown" | "pgdown" | "pgdn" => Key::PageDown,
        "escape" | "esc" => Key::Escape,
        "plus" => Key::Char('+'),
        "minus" => Key::Char('-'),
        _ => {
            let mut chars = name.chars();
            if let Some(ch) = chars.next() {
                if chars.next().is_none() {
                    return Ok(Key::Char(ch.to_ascii_lowercase()));
                }
            }
            if let Some(number) = lower.strip_prefix('f') {
                if let Ok(number) = number.parse::<u8>() {
                    if (1..=24).contains(&number) {
                        return Ok(Key::F(number));
                    }
                }
            }
            return Err(ShortcutError::UnknownKey(name.to_owned()));
        }
    };
    Ok(key)
}

/// Returns the display name of a shortcut key.
fn key_label(key: Key) -> String {
    match key {
        Key::Backspace => "Backspace".into(),
        Key::Delete => "Delete".into(),
        Key::Insert => "Insert".into(),
        Key::Left => "Left".into(),
        Key::Right => "Right".into(),
        Key::Up => "Up".into(),
        Key::Down => "Down".into(),
        Key::Home => "Home".into(),
        Key::End => "End".into(),
        Key::Tab => "Tab".into(),
        Key::ShiftTab => "ShiftTab".into(),
        Key::Enter => "Enter".into(),
        Key::Space => "Space".into(),
        Key::PageUp => "PageUp".into(),
        Key::PageDown => "PageDown".into(),
        Key::Escape => "Escape".into(),
        Key::Char(ch) => ch.to_ascii_uppercase().to_string(),
        Key::F(number) => format!("F{number}"),
    }
}
