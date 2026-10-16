use std::any::Any;

use crate::layout::{self, GridItem, GridOpts, PackItem, PackOpts, Rect, Size};
use crate::render::TextEngine;
use crate::widgets::{Binding, Kind, Manager, WidgetNode};
use crate::{Opt, WidgetId};

/// Owns the widget arena, variables, callbacks, and text renderer.
///
/// All public mutation happens through `&mut Tk`; handles contain no shared
/// mutable pointers. Widgets are retained until this `Tk` is dropped.
pub struct Tk {
    pub(crate) nodes: Vec<WidgetNode>,
    pub(crate) vars: Vec<Box<dyn Any>>,
    pub(crate) text_engine: TextEngine,
    pub(crate) dirty: bool,
    pub(crate) window_title: String,
    pub(crate) window_size: Option<(u32, u32)>,
    pub(crate) quit_requested: bool,
    pub(crate) focused: Option<WidgetId>,
    pub(crate) hovered: Option<WidgetId>,
    pub(crate) pressed: Option<WidgetId>,
}

impl Default for Tk {
    fn default() -> Self {
        Self::new()
    }
}

impl Tk {
    pub fn new() -> Self {
        Self {
            nodes: vec![WidgetNode::new(Kind::Frame, None, String::new())],
            vars: Vec::new(),
            text_engine: TextEngine::new(),
            dirty: true,
            window_title: "rtk".into(),
            window_size: None,
            quit_requested: false,
            focused: None,
            hovered: None,
            pressed: None,
        }
    }

    pub fn root(&self) -> WidgetId {
        WidgetId(0)
    }

    pub fn title(&mut self, title: impl Into<String>) {
        self.window_title = title.into();
        self.dirty = true;
    }

    /// Overrides the initial window size, in physical pixels.
    pub fn geometry(&mut self, width: u32, height: u32) {
        assert!(width > 0 && height > 0, "geometry must be nonzero");
        self.window_size = Some((width, height));
        self.dirty = true;
    }

    pub fn quit(&mut self) {
        self.quit_requested = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn add(&mut self, kind: Kind, parent: WidgetId, text: String) -> WidgetId {
        assert_eq!(self.nodes[parent.0 as usize].kind, Kind::Frame, "parent must be a frame");
        let id = WidgetId(u32::try_from(self.nodes.len()).expect("widget arena is full"));
        self.nodes.push(WidgetNode::new(kind, Some(parent), text));
        self.nodes[parent.0 as usize].children.push(id);
        self.dirty = true;
        id
    }

    pub fn config(&mut self, id: WidgetId, option: Opt) {
        let index = id.0 as usize;
        match option {
            Opt::Text(text) => {
                if self.nodes[index].kind == Kind::Entry {
                    self.entry_set(id, text);
                } else {
                    self.nodes[index].text = text;
                }
            }
            Opt::Bg(color) => self.nodes[index].style.bg = color,
            Opt::Fg(color) => self.nodes[index].style.fg = color,
            Opt::Width(chars) => self.nodes[index].style.width = Some(chars),
            Opt::FontSize(size) => {
                assert!(size.is_finite() && size > 0.0, "font size must be positive and finite");
                self.nodes[index].style.font_size = size;
            }
            Opt::TextVariable(var) => {
                self.nodes[index].binding = Some(Binding::Text(var));
                self.nodes[index].cursor = var.get(self).len();
            }
            Opt::TextVariableInt(var) => {
                assert_ne!(self.nodes[index].kind, Kind::Entry, "entries need string variables");
                self.nodes[index].binding = Some(Binding::Int(var));
            }
            Opt::Command(command) => {
                assert_eq!(self.nodes[index].kind, Kind::Button, "commands require buttons");
                self.nodes[index].command = Some(command);
            }
        }
        self.dirty = true;
    }

    pub fn set_command(&mut self, id: WidgetId, command: impl FnMut(&mut Tk) + 'static) {
        self.config(id, Opt::Command(Box::new(command)));
    }

    pub fn entry_get(&self, id: WidgetId) -> String {
        assert_eq!(self.nodes[id.0 as usize].kind, Kind::Entry, "not an entry");
        self.display_text(id)
    }

    pub fn entry_set(&mut self, id: WidgetId, text: impl Into<String>) {
        assert_eq!(self.nodes[id.0 as usize].kind, Kind::Entry, "not an entry");
        let text = text.into();
        // Entries are single-line, including when set programmatically.
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        self.nodes[id.0 as usize].cursor = text.len();
        self.write_entry(id, text);
    }

    fn write_entry(&mut self, id: WidgetId, text: String) {
        if let Some(Binding::Text(var)) = self.nodes[id.0 as usize].binding {
            var.set(self, text.clone());
        }
        self.nodes[id.0 as usize].text = text;
        self.dirty = true;
    }

    pub(crate) fn display_text(&self, id: WidgetId) -> String {
        let node = &self.nodes[id.0 as usize];
        match node.binding {
            Some(Binding::Text(var)) => var.get(self),
            Some(Binding::Int(var)) => var.get(self).to_string(),
            None => node.text.clone(),
        }
    }

    fn check_manager(&self, id: WidgetId, pack: bool) {
        let parent = self.nodes[id.0 as usize].parent.expect("cannot manage the root");
        for &sibling in &self.nodes[parent.0 as usize].children {
            if sibling != id {
                if let Some(manager) = self.nodes[sibling.0 as usize].manager {
                    assert_eq!(
                        matches!(manager, Manager::Pack(_)),
                        pack,
                        "cannot mix pack and grid in one parent"
                    );
                }
            }
        }
    }

    pub fn pack(&mut self, id: WidgetId, options: PackOpts) {
        self.check_manager(id, true);
        self.nodes[id.0 as usize].manager = Some(Manager::Pack(options));
        self.dirty = true;
    }

    pub fn grid(&mut self, id: WidgetId, options: GridOpts) {
        self.check_manager(id, false);
        self.nodes[id.0 as usize].manager = Some(Manager::Grid(options));
        self.dirty = true;
    }

    pub fn grid_columnconfigure(&mut self, parent: WidgetId, column: usize, weight: u32) {
        let node = &mut self.nodes[parent.0 as usize];
        assert_eq!(node.kind, Kind::Frame, "grid parent must be a frame");
        node.columns.insert(column, weight);
        self.dirty = true;
    }

    pub fn grid_rowconfigure(&mut self, parent: WidgetId, row: usize, weight: u32) {
        let node = &mut self.nodes[parent.0 as usize];
        assert_eq!(node.kind, Kind::Frame, "grid parent must be a frame");
        node.rows.insert(row, weight);
        self.dirty = true;
    }

    fn managed_children(&self, id: WidgetId) -> Vec<WidgetId> {
        self.nodes[id.0 as usize]
            .children
            .iter()
            .copied()
            .filter(|child| self.nodes[child.0 as usize].manager.is_some())
            .collect()
    }

    fn pack_items(&self, children: &[WidgetId]) -> Vec<PackItem> {
        children.iter().filter_map(|id| {
            let node = &self.nodes[id.0 as usize];
            match node.manager {
                Some(Manager::Pack(opts)) => Some(PackItem { requested: node.requested, opts }),
                _ => None,
            }
        }).collect()
    }

    fn grid_items(&self, children: &[WidgetId]) -> Vec<GridItem> {
        children.iter().filter_map(|id| {
            let node = &self.nodes[id.0 as usize];
            match node.manager {
                Some(Manager::Grid(opts)) => Some(GridItem { requested: node.requested, opts }),
                _ => None,
            }
        }).collect()
    }

    fn update_requested(&mut self, id: WidgetId) -> Size {
        let children = self.managed_children(id);
        for &child in &children {
            self.update_requested(child);
        }
        let index = id.0 as usize;
        let kind = self.nodes[index].kind;
        let style = self.nodes[index].style;
        let mut requested = if kind == Kind::Frame {
            let mut size = if children.first().is_some_and(|child| {
                matches!(self.nodes[child.0 as usize].manager, Some(Manager::Pack(_)))
            }) {
                layout::pack_request(&self.pack_items(&children))
            } else {
                layout::grid_request(&self.grid_items(&children))
            };
            if self.nodes[index].parent.is_some() {
                size.width += 4.0;
                size.height += 4.0;
            }
            size
        } else {
            let text = self.display_text(id);
            let measured = self.text_engine.measure(&text, style.font_size);
            let char_width = self.text_engine.measure("0", style.font_size).width
                .max(style.font_size * 0.5);
            let text_width = style.width.map(|chars| chars as f32 * char_width);
            match kind {
                Kind::Label => Size::new(text_width.unwrap_or(measured.width) + 12.0, measured.height + 8.0),
                Kind::Button => Size::new(text_width.unwrap_or(measured.width) + 20.0, measured.height + 12.0),
                Kind::Entry => Size::new(
                    text_width.unwrap_or(20.0 * char_width) + 12.0,
                    style.font_size * 1.3 + 12.0,
                ),
                Kind::Frame => unreachable!(),
            }
        };
        if kind == Kind::Frame {
            if let Some(chars) = style.width {
                let char_width = self.text_engine.measure("0", style.font_size).width
                    .max(style.font_size * 0.5);
                requested.width = requested.width.max(chars as f32 * char_width);
            }
        }
        self.nodes[index].requested = requested;
        requested
    }

    pub(crate) fn initial_size(&mut self) -> (u32, u32) {
        if let Some(size) = self.window_size {
            return size;
        }
        let requested = self.update_requested(self.root());
        (
            requested.width.ceil().max(200.0) as u32,
            requested.height.ceil().max(100.0) as u32,
        )
    }

    pub(crate) fn layout(&mut self, width: u32, height: u32) {
        self.update_requested(self.root());
        for node in &mut self.nodes {
            node.visible = false;
        }
        let rect = Rect::new(0.0, 0.0, width as f32, height as f32);
        self.place_tree(self.root(), rect, rect);
    }

    fn place_tree(&mut self, id: WidgetId, rect: Rect, parent_clip: Rect) {
        let index = id.0 as usize;
        let clip = rect.intersection(parent_clip);
        let node = &mut self.nodes[index];
        node.rect = rect;
        node.clip = clip;
        node.visible = true;
        let content = if node.parent.is_some() { rect.inset(2.0, 2.0) } else { rect };
        let children = self.managed_children(id);
        let Some(first) = children.first() else { return };
        let rects = if matches!(self.nodes[first.0 as usize].manager, Some(Manager::Pack(_))) {
            layout::pack(content, &self.pack_items(&children))
        } else {
            let columns: Vec<_> = self.nodes[index].columns.iter().map(|(&i, &w)| (i, w)).collect();
            let rows: Vec<_> = self.nodes[index].rows.iter().map(|(&i, &w)| (i, w)).collect();
            layout::grid(content, &self.grid_items(&children), &columns, &rows)
        };
        for (child, child_rect) in children.into_iter().zip(rects) {
            self.place_tree(child, child_rect, clip.intersection(content));
        }
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<WidgetId> {
        self.nodes.iter().enumerate().rev().find_map(|(index, node)| {
            (node.visible
                && matches!(node.kind, Kind::Button | Kind::Entry)
                && node.rect.contains(x, y)
                && node.clip.contains(x, y))
                .then_some(WidgetId(index as u32))
        })
    }

    pub(crate) fn pointer_move(&mut self, x: f32, y: f32) {
        let hovered = self.hit_test(x, y)
            .filter(|id| self.nodes[id.0 as usize].kind == Kind::Button);
        if hovered != self.hovered {
            self.hovered = hovered;
            self.dirty = true;
        }
    }

    pub(crate) fn pointer_press(&mut self, x: f32, y: f32) {
        self.pointer_move(x, y);
        let hit = self.hit_test(x, y);
        self.pressed = hit.filter(|id| self.nodes[id.0 as usize].kind == Kind::Button);
        if let Some(id) = hit.filter(|id| self.nodes[id.0 as usize].kind == Kind::Entry) {
            self.focused = Some(id);
            let text = self.display_text(id);
            let node = &self.nodes[id.0 as usize];
            let target = (x - node.rect.x - 6.0 + node.scroll_x).max(0.0);
            let font_size = node.style.font_size;
            let (mut best, mut distance) = (0, target);
            for (start, ch) in text.char_indices() {
                let end = start + ch.len_utf8();
                let width = self.text_engine.measure(&text[..end], font_size).width;
                let next_distance = (width - target).abs();
                if next_distance < distance {
                    best = end;
                    distance = next_distance;
                }
            }
            self.nodes[id.0 as usize].cursor = best;
        } else if hit.is_none() {
            self.focused = None;
        }
        self.dirty = true;
    }

    pub(crate) fn pointer_release(&mut self, x: f32, y: f32) {
        if let Some(id) = self.pressed.take() {
            self.dirty = true;
            if self.hit_test(x, y) == Some(id) {
                // No arena borrow survives the callback. A callback may replace
                // its own command; in that case the replacement wins.
                if let Some(mut command) = self.nodes[id.0 as usize].command.take() {
                    command(self);
                    if self.nodes[id.0 as usize].command.is_none() {
                        self.nodes[id.0 as usize].command = Some(command);
                    }
                }
            }
        }
    }

    pub(crate) fn edit(&mut self, edit: Edit) {
        let Some(id) = self.focused else { return };
        let mut text = self.display_text(id);
        let mut cursor = cursor_boundary(&text, self.nodes[id.0 as usize].cursor);
        let mut changed = false;
        match edit {
            Edit::Left => cursor = previous_boundary(&text, cursor),
            Edit::Right => {
                cursor += text[cursor..].chars().next().map_or(0, char::len_utf8);
            }
            Edit::Home => cursor = 0,
            Edit::End => cursor = text.len(),
            Edit::Backspace if cursor > 0 => {
                let previous = previous_boundary(&text, cursor);
                text.replace_range(previous..cursor, "");
                cursor = previous;
                changed = true;
            }
            Edit::Delete if cursor < text.len() => {
                let end = cursor + text[cursor..].chars().next().unwrap().len_utf8();
                text.replace_range(cursor..end, "");
                changed = true;
            }
            Edit::Insert(value) => {
                let value: String = value.chars().filter(|c| !c.is_control()).collect();
                if !value.is_empty() {
                    text.insert_str(cursor, &value);
                    cursor += value.len();
                    changed = true;
                }
            }
            Edit::Backspace | Edit::Delete => {}
        }
        if changed {
            self.write_entry(id, text);
        }
        self.nodes[id.0 as usize].cursor = cursor;
        self.dirty = true;
    }
}

pub(crate) enum Edit {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Insert(String),
}

pub(crate) fn cursor_boundary(text: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(text.len());
    while !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

fn previous_boundary(text: &str, cursor: usize) -> usize {
    text[..cursor].char_indices().next_back().map_or(0, |(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::{Edit, Tk};
    use crate::{Entry, Opt};

    #[test]
    fn unicode_editing_and_external_binding_changes() {
        let mut tk = Tk::new();
        let root = tk.root();
        let value = tk.string_var("aé🙂");
        let entry = Entry::new(&mut tk, root);
        tk.config(entry, Opt::TextVariable(value));
        tk.focused = Some(entry);
        tk.edit(Edit::Left);
        tk.edit(Edit::Backspace);
        assert_eq!(value.get(&tk), "a🙂");
        tk.edit(Edit::Delete);
        assert_eq!(value.get(&tk), "a");
        value.set(&mut tk, String::new());
        tk.edit(Edit::Insert("λ".into()));
        assert_eq!(tk.entry_get(entry), "λ");
        tk.edit(Edit::Home);
        tk.edit(Edit::Delete);
        assert_eq!(tk.entry_get(entry), "");
    }
}
