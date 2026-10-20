//! AccessKit integration: translates the toolkit's accessibility tree into
//! AccessKit `TreeUpdate`s and maps incoming action requests back to widgets.
//!
//! Compiled only with the `accesskit` feature.

use std::collections::{HashMap, HashSet};

use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Rect as AkRect, Role, Toggled, TreeId,
    TreeInfo, TreeUpdate,
};

use crate::a11y::{A11yAction, A11yRole};
use crate::tk::Tk;
use crate::WidgetId;

/// Builds an AccessKit [`TreeUpdate`] from the toolkit's accessibility tree.
///
/// `scale` converts logical-pixel bounds into physical pixels.
pub(crate) fn tree_update(tk: &Tk, scale: f64) -> TreeUpdate {
    let nodes = tk.accessibility_tree();
    let root = tk.root();
    let root_id = NodeId(root.a11y_key());
    let mut map: HashMap<NodeId, Node> = HashMap::new();
    let mut order: Vec<NodeId> = Vec::new();
    let present: HashSet<NodeId> = nodes.iter().map(|n| NodeId(n.id.a11y_key())).collect();
    for a11y in &nodes {
        let node_id = NodeId(a11y.id.a11y_key());
        let mut node = Node::new(map_role(a11y.role));
        if let Some(label) = &a11y.label {
            node.set_label(label.clone());
        }
        if let Some(value) = &a11y.value {
            node.set_value(value.clone());
        }
        if let Some((value, min, max)) = a11y.numeric {
            node.set_numeric_value(value);
            node.set_min_numeric_value(min);
            node.set_max_numeric_value(max);
        }
        if let Some(toggled) = a11y.toggled {
            node.set_toggled(if toggled { Toggled::True } else { Toggled::False });
        }
        if a11y.disabled {
            node.set_disabled();
        }
        let bounds = a11y.bounds;
        node.set_bounds(AkRect {
            x0: f64::from(bounds.x) * scale,
            y0: f64::from(bounds.y) * scale,
            x1: f64::from(bounds.x + bounds.width) * scale,
            y1: f64::from(bounds.y + bounds.height) * scale,
        });
        let children: Vec<NodeId> = a11y
            .children
            .iter()
            .map(|c| NodeId(c.a11y_key()))
            .filter(|c| present.contains(c))
            .collect();
        node.set_children(children);
        for action in &a11y.actions {
            if let Some(action) = map_action_kind(*action) {
                node.add_action(action);
            }
        }
        if a11y.id == root {
            node.set_label(tk.window_title.clone());
        }
        map.insert(node_id, node);
        order.push(node_id);
    }
    let mut tree_info = TreeInfo::new(root_id);
    tree_info.toolkit_name = Some("rtk".into());
    tree_info.toolkit_version = Some(env!("CARGO_PKG_VERSION").into());
    let focus = tk
        .focused_widget()
        .map(|id| NodeId(id.a11y_key()))
        .filter(|id| present.contains(id))
        .unwrap_or(root_id);
    let nodes: Vec<(NodeId, Node)> = order
        .into_iter()
        .filter_map(|id| map.remove(&id).map(|node| (id, node)))
        .collect();
    TreeUpdate {
        nodes,
        tree: Some(tree_info),
        tree_id: TreeId::ROOT,
        focus,
    }
}

/// Maps an [`A11yRole`] to an AccessKit [`Role`].
fn map_role(role: A11yRole) -> Role {
    match role {
        A11yRole::Window => Role::Window,
        A11yRole::Group => Role::GenericContainer,
        A11yRole::Label => Role::Label,
        A11yRole::Button => Role::Button,
        A11yRole::CheckBox => Role::CheckBox,
        A11yRole::RadioButton => Role::RadioButton,
        A11yRole::TextInput => Role::TextInput,
        A11yRole::MultilineTextInput => Role::MultilineTextInput,
        A11yRole::Slider => Role::Slider,
        A11yRole::SpinButton => Role::SpinButton,
        A11yRole::ProgressIndicator => Role::ProgressIndicator,
        A11yRole::ListBox => Role::ListBox,
        A11yRole::ComboBox => Role::ComboBox,
        A11yRole::TabList => Role::TabList,
        A11yRole::ScrollBar => Role::ScrollBar,
        A11yRole::Canvas => Role::Canvas,
        A11yRole::Unknown => Role::GenericContainer,
    }
}

/// Maps an [`A11yAction`] to an AccessKit [`Action`].
fn map_action_kind(action: A11yAction) -> Option<Action> {
    match action {
        A11yAction::Click => Some(Action::Click),
        A11yAction::Focus => Some(Action::Focus),
        A11yAction::SetValue => Some(Action::SetValue),
        A11yAction::Increment => Some(Action::Increment),
        A11yAction::Decrement => Some(Action::Decrement),
        // AccessKit has no dedicated toggle action; screen readers toggle
        // check boxes and switches by issuing a Click.
        A11yAction::Toggle => Some(Action::Click),
    }
}

/// Maps an incoming AccessKit [`ActionRequest`] to a widget key, action and
/// optional string value.
pub(crate) fn map_action(req: &ActionRequest) -> Option<(u64, A11yAction, Option<String>)> {
    let action = match req.action {
        Action::Click => A11yAction::Click,
        Action::Focus => A11yAction::Focus,
        Action::SetValue => A11yAction::SetValue,
        Action::Increment => A11yAction::Increment,
        Action::Decrement => A11yAction::Decrement,
        _ => return None,
    };
    let value = match &req.data {
        Some(ActionData::Value(v)) => Some(v.to_string()),
        Some(ActionData::NumericValue(n)) => Some(n.to_string()),
        _ => None,
    };
    Some((req.target_node.0, action, value))
}

/// Resolves a widget key produced by [`WidgetId::a11y_key`] back to a handle.
pub(crate) fn widget_from_key(key: u64) -> WidgetId {
    WidgetId::from_a11y_key(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::{Button, Checkbutton, Entry, Frame, Label, ProgressBar, Scale};

    /// Builds a small scene and checks the tree update node count and roles.
    #[test]
    fn tree_update_has_expected_nodes() {
        let mut tk = Tk::new();
        let root = tk.root();
        let frame = Frame::new(&mut tk, root).unwrap();
        let label = Label::new(&mut tk, frame, "Hello").unwrap();
        let button = Button::new(&mut tk, frame, "Go").unwrap();
        let entry = Entry::new(&mut tk, frame).unwrap();
        let var = tk.bool_var(true);
        let check = Checkbutton::new(&mut tk, frame, "On", var).unwrap();
        let scale_var = tk.int_var(5);
        let scale = Scale::new(&mut tk, frame, 0, 10, scale_var).unwrap();
        let prog_var = tk.int_var(50);
        let prog = ProgressBar::new(&mut tk, frame, 100.0, prog_var).unwrap();
        tk.pack(frame, crate::PackOpts::default());
        for id in [label, button, entry, check, scale, prog] {
            tk.pack(id, crate::PackOpts::default());
        }
        tk.sim_resize(640, 480);
        let update = tree_update(&tk, 1.0);
        assert!(update.nodes.len() >= 8);
        let root_node = update.nodes.iter().find(|(id, _)| *id == NodeId(root.a11y_key()));
        assert!(root_node.is_some());
        let roles: Vec<Role> = update.nodes.iter().map(|(_, n)| n.role()).collect();
        assert!(roles.contains(&Role::Button));
        assert!(roles.contains(&Role::CheckBox));
        assert!(roles.contains(&Role::Slider));
        assert!(roles.contains(&Role::ProgressIndicator));
    }

    /// Checks that action requests map to the expected toolkit action.
    #[test]
    fn map_action_translates_requests() {
        let req = ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: NodeId(42),
            data: None,
        };
        assert_eq!(map_action(&req), Some((42, A11yAction::Click, None)));
        let req = ActionRequest {
            action: Action::SetValue,
            target_tree: TreeId::ROOT,
            target_node: NodeId(7),
            data: Some(ActionData::Value("hi".into())),
        };
        assert_eq!(map_action(&req), Some((7, A11yAction::SetValue, Some("hi".into()))));
        let req = ActionRequest {
            action: Action::ScrollDown,
            target_tree: TreeId::ROOT,
            target_node: NodeId(1),
            data: None,
        };
        assert_eq!(map_action(&req), None);
    }
}
