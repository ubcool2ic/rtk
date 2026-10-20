//! Styling cascade, themes, tokens, images and animation.

#[allow(unused_imports)]
use super::*;

pub(crate) fn kind_class_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Frame => "TFrame",
        Kind::Label => "TLabel",
        Kind::Button => "TButton",
        Kind::Entry => "TEntry",
        Kind::Checkbutton => "TCheckbutton",
        Kind::Scale => "TScale",
        Kind::Listbox => "TListbox",
        Kind::Scrollbar => "TScrollbar",
        Kind::Radiobutton => "TRadiobutton",
        Kind::Canvas => "TCanvas",
        Kind::Spinbox => "TSpinbox",
        Kind::ProgressBar => "TProgressbar",
        Kind::Text => "TText",
        Kind::Combobox => "TCombobox",
        Kind::Notebook => "TNotebook",
        Kind::Table => "TTable",
        Kind::TreeView => "TTreeview",
        Kind::Menubar => "TMenubar",
    }
}


pub(crate) struct NodeSnapshot {
    pub(crate) scale: Option<crate::Var<i64>>,
    pub(crate) range: (i64, i64),
    pub(crate) canvas_size: (f32, f32),
    pub(crate) progress_length: f32,
    pub(crate) list_height: usize,
    pub(crate) parent: Option<WidgetId>,
    pub(crate) kind: Kind,
    pub(crate) style: crate::widgets::Style,
    pub(crate) image: Option<ImageId>,
}

/// Legacy (pre-style-engine) dark-theme background/foreground for a widget
/// kind. Shared by `set_theme` and widget creation so both agree (review F9).
pub(crate) fn dark_default_colors(kind: Kind) -> (Color, Color) {
    let bg = match kind {
        Kind::Entry | Kind::Listbox | Kind::Text | Kind::Combobox => Color::rgb(23, 26, 33),
        Kind::Canvas => Color::rgb(20, 22, 28),
        _ => Color::rgb(23, 26, 33),
    };
    (bg, Color::rgb(230, 232, 238))
}

impl Tk {
    // -----------------------------------------------------------------------
    // Style API
    // -----------------------------------------------------------------------

    /// ttk `style.configure(name, **opts)`: sets the base spec of a named style.
    pub fn style_configure(&mut self, style_name: impl Into<String>, spec: StyleSpec) {
        self.style_engine.configure(style_name, spec);
        self.dirty = true;
    }

    /// ttk `style.map(name, state=[...])`: sets the spec applied while `state` is active.
    pub fn style_map(&mut self, style_name: impl Into<String>, state: State, spec: StyleSpec) {
        self.style_engine.map(style_name, state, spec);
        self.dirty = true;
    }

    /// Parses a CSS-like stylesheet into the style engine, returning the rule count.
    pub fn load_stylesheet(&mut self, css: &str) -> Result<usize, StyleError> {
        let parser = StylesheetParser::new(css);
        let count = parser.parse(&mut self.style_engine)?;
        self.dirty = true;
        Ok(count)
    }

    /// Reads and parses a stylesheet file; I/O errors are reported as `StyleError` at line 0.
    pub fn load_stylesheet_file(&mut self, path: impl AsRef<Path>) -> Result<usize, StyleError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| StyleError {
            line: 0,
            message: format!("cannot read file {}: {e}", path.display()),
        })?;
        self.load_stylesheet(&content)
    }

    /// Loads a stylesheet and remembers its mtime so [`Tk::poll_stylesheet`] can hot-reload it.
    /// Returns false if the file cannot be stat'ed.
    pub fn watch_stylesheet(&mut self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref().to_path_buf();
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                let _ = self.load_stylesheet_file(&path);
                self.watched_stylesheet = Some((path, mtime));
                return true;
            }
        }
        false
    }

    /// Reloads the watched stylesheet if its mtime advanced. Returns true when reloaded.
    pub fn poll_stylesheet(&mut self) -> bool {
        let Some((path, last_mtime)) = self.watched_stylesheet.clone() else { return false };
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                if mtime > last_mtime {
                    self.watched_stylesheet = Some((path.clone(), mtime));
                    let _ = self.load_stylesheet_file(&path);
                    return true;
                }
            }
        }
        false
    }

    /// Loads a PNG, either from a file path (UTF-8 string that names a readable file)
    /// or from in-memory PNG bytes. Returns `None` if decoding fails.
    pub fn load_image(&mut self, path_or_png_bytes: impl AsRef<[u8]>) -> Option<ImageId> {
        let bytes = path_or_png_bytes.as_ref();
        let pixmap = if let Ok(s) = std::str::from_utf8(bytes) {
            if let Ok(file_bytes) = fs::read(s) {
                Pixmap::decode_png(&file_bytes).ok()
            } else {
                Pixmap::decode_png(bytes).ok()
            }
        } else {
            Pixmap::decode_png(bytes).ok()
        }?;
        let id = ImageId(self.next_image_id);
        self.next_image_id = self.next_image_id.saturating_add(1);
        self.images.insert(id, pixmap);
        Some(id)
    }

    /// Registers a raw RGBA buffer as an image, premultiplying color channels by alpha.
    pub fn register_image_rgba(&mut self, w: u32, h: u32, rgba: &[u8]) -> Option<ImageId> {
        if w == 0 || h == 0 || rgba.len() != (w as usize) * (h as usize) * 4 {
            return None;
        }
        let mut pixmap = Pixmap::new(w, h)?;
        let data = pixmap.data_mut();
        for (dst, src) in data.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
            let a = u32::from(src[3]);
            dst[0] = (u32::from(src[0]) * a / 255) as u8;
            dst[1] = (u32::from(src[1]) * a / 255) as u8;
            dst[2] = (u32::from(src[2]) * a / 255) as u8;
            dst[3] = src[3];
        }
        let id = ImageId(self.next_image_id);
        self.next_image_id = self.next_image_id.saturating_add(1);
        self.images.insert(id, pixmap);
        Some(id)
    }

    /// Current ttk-style state flags (hover/pressed/focused/checked/...) of a widget.
    pub fn widget_state(&self, id: WidgetId) -> State {
        let Some(node) = self.node(id) else { return State::empty() };
        let mut s = State::empty();
        if node.disabled {
            s |= State::DISABLED;
        }
        if self.hovered == Some(id) {
            s |= State::HOVER;
        }
        if self.pressed == Some(id) {
            s |= State::PRESSED;
        }
        if self.focused == Some(id) {
            s |= State::FOCUSED;
        }
        if node.selected {
            s |= State::SELECTED;
        }
        if node.kind == Kind::Checkbutton {
            if node.check.and_then(|v| v.get(self)).unwrap_or(false) {
                s |= State::CHECKED;
            }
        } else if node.kind == Kind::Radiobutton {
            if node.radio_var.and_then(|v| v.get(self)) == Some(node.radio_value) {
                s |= State::CHECKED;
            }
        } else if node.kind == Kind::Listbox && node.list_selection.is_some() {
            s |= State::SELECTED;
        }
        s
    }

    /// Background colour currently shown, including an in-flight transition.
    #[doc(hidden)]
    pub fn displayed_bg(&self, id: WidgetId) -> Option<Color> {
        let node = self.node(id)?;
        match node.transition.as_ref() {
            Some(transition) => Some(transition.current_bg()),
            None => self.resolved_style(id).map(|style| style.bg),
        }
    }

    /// Resolves the concrete style for widget `id` by cascading:
    /// theme default -> "TClass" -> "Named.TClass" -> Frame inheritance -> widget override -> state map -> legacy Opt.
    pub fn resolved_style(&self, id: WidgetId) -> Option<ResolvedStyle> {
        self.resolved_style_in(id, self.widget_state(id))
    }

    /// Resolves the cascade as if the widget were in `state`.
    pub(crate) fn resolved_style_in(&self, id: WidgetId, state: State) -> Option<ResolvedStyle> {
        let node = self.node(id)?;
        let kind = node.kind;
        let class_name = kind_class_name(kind);
        let mut res = ResolvedStyle::default();

        // 1. Theme default for class
        if let Some(spec) = self.style_engine.lookup(class_name) {
            spec.merge_into(&mut res);
        }

        // 2. Named style if specified on node
        if let Some(named) = &node.named_style {
            if named != class_name {
                if let Some(spec) = self.style_engine.lookup(named) {
                    spec.merge_into(&mut res);
                }
            }
        }

        // 3. Inherited properties from ancestor Frames (fg, font_family, font_size, font_weight, italic)
        let mut ancestor = node.parent;
        let mut inherited_fg = None;
        let mut inherited_font_family = None;
        let mut inherited_font_size = None;
        let mut inherited_font_weight = None;
        let mut inherited_italic = None;

        while let Some(pid) = ancestor {
            if let Some(pnode) = self.node(pid) {
                if let Some(named) = &pnode.named_style {
                    if let Some(spec) = self.style_engine.lookup(named) {
                        if inherited_fg.is_none() { inherited_fg = spec.fg; }
                        if inherited_font_family.is_none() { inherited_font_family = spec.font_family.clone(); }
                        if inherited_font_size.is_none() { inherited_font_size = spec.font_size; }
                        if inherited_font_weight.is_none() { inherited_font_weight = spec.font_weight; }
                        if inherited_italic.is_none() { inherited_italic = spec.italic; }
                    }
                }
                if let Some(ovr) = &pnode.style_override {
                    if inherited_fg.is_none() { inherited_fg = ovr.fg; }
                    if inherited_font_family.is_none() { inherited_font_family = ovr.font_family.clone(); }
                    if inherited_font_size.is_none() { inherited_font_size = ovr.font_size; }
                    if inherited_font_weight.is_none() { inherited_font_weight = ovr.font_weight; }
                    if inherited_italic.is_none() { inherited_italic = ovr.italic; }
                }
                ancestor = pnode.parent;
            } else {
                break;
            }
        }
        if let Some(c) = inherited_fg { res.fg = c; }
        if let Some(ff) = inherited_font_family { res.font_family = ff; }
        if let Some(fs) = inherited_font_size { res.font_size = fs; }
        if let Some(fw) = inherited_font_weight { res.font_weight = fw; }
        if let Some(it) = inherited_italic { res.italic = it; }

        // 4. Per-widget override
        if let Some(ovr) = &node.style_override {
            ovr.merge_into(&mut res);
        }

        if node.monospace {
            res.font_family = Some("monospace".into());
        }

        // 5. State maps for the requested states
        let style_key = node.named_style.as_deref().unwrap_or(class_name);
        for check_state in [
            State::HOVER, State::FOCUSED, State::PRESSED,
            State::CHECKED, State::SELECTED, State::DISABLED,
        ] {
            if state.contains(check_state) {
                // Check specific named style then fallback to class
                if let Some(spec) = self.style_engine.lookup_state(style_key, check_state) {
                    spec.merge_into(&mut res);
                } else if style_key != class_name {
                    if let Some(spec) = self.style_engine.lookup_state(class_name, check_state) {
                        spec.merge_into(&mut res);
                    }
                }
            }
        }

        // 6. Legacy Opt::Bg/Fg/FontSize wins if explicitly set
        if node.style.bg_overridden {
            res.bg = node.style.bg;
        }
        if node.style.fg_overridden {
            res.fg = node.style.fg;
        }
        if node.style.font_size != 14.0 {
            res.font_size = node.style.font_size;
        }

        Some(res)
    }

    /// Advances simulated animation time by `ms` milliseconds for transitions.
    pub fn sim_advance_time(&mut self, ms: u32) {
        if self.hovered.is_some() {
            self.hover_timer_ms = self.hover_timer_ms.saturating_add(ms);
            if self.hover_timer_ms >= 500 && !self.tooltip_visible {
                let has_tip = self.hovered
                    .and_then(|id| self.node(id))
                    .and_then(|n| n.tooltip.as_ref())
                    .is_some();
                if has_tip {
                    self.tooltip_visible = true;
                    self.dirty = true;
                }
            }
        }

        for slot in &mut self.nodes {
            if let Some(node) = &mut slot.node {
                if let Some(trans) = &mut node.transition {
                    let done = trans.advance(ms);
                    if done {
                        node.transition = None;
                    }
                    self.dirty = true;
                }
                if node.kind == Kind::ProgressBar && node.progress_indeterminate && node.visible {
                    node.progress_phase = (node.progress_phase + (ms as f32) / 1000.0) % 1.0;
                    self.dirty = true;
                }
            }
        }

        if self.toasts.advance(ms) {
            self.dirty = true;
        }
    }

    /// True while any style transition, indeterminate progress bar or toast needs frames.
    pub fn is_animating(&self) -> bool {
        self.toasts.is_animating()
            || self.nodes.iter().filter_map(|s| s.node.as_ref()).any(|n| {
                n.transition.is_some() || (n.kind == Kind::ProgressBar && n.progress_indeterminate && n.visible)
            })
    }


    pub(crate) fn check_state_transition(&mut self, id: WidgetId, old_state: State) {
        let new_state = self.widget_state(id);
        if old_state == new_state {
            return;
        }
        let Some(resolved) = self.resolved_style_in(id, new_state) else { return };
        let Some(previous) = self.resolved_style_in(id, old_state) else { return };
        if resolved.transition_ms > 0 {
            if let Some(node) = self.node_mut(id) {
                // Start from what is on screen: an in-flight transition, else the old state's style.
                let from_bg = node.transition.as_ref().map(|t| t.current_bg()).unwrap_or(previous.bg);
                let from_fg = node.transition.as_ref().map(|t| t.current_fg()).unwrap_or(previous.fg);
                let from_border = node.transition.as_ref().map(|t| t.current_border()).unwrap_or(previous.border_color);
                node.transition = Some(TransitionState::new(
                    from_bg, resolved.bg,
                    from_fg, resolved.fg,
                    from_border, resolved.border_color,
                    resolved.transition_ms,
                ));
            }
        }
    }

}
