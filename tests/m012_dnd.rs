//! Integration tests for file drag-and-drop bindings (milestone 0.12, step 3).

use std::path::PathBuf;

use rtk::{DropPhase, Fill, Frame, PackOpts, Tk, WidgetId};

/// Builds a toolkit with a full-viewport outer frame and a nested inner frame,
/// lays it out, and returns `(tk, outer, inner)`.
fn nested_frames() -> (Tk, WidgetId, WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let outer = Frame::new(&mut tk, root).expect("outer frame");
    tk.pack(
        outer,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    let inner = Frame::new(&mut tk, outer).expect("inner frame");
    tk.pack(
        inner,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    tk.sim_resize(400, 300);
    (tk, outer, inner)
}

/// Returns the centre point of a widget's rect.
fn centre(tk: &mut Tk, id: WidgetId) -> (f32, f32) {
    let rect = tk.widget_rect(id).expect("widget has a rect");
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

/// A drop on a bound child reaches that child.
#[test]
fn drop_on_bound_child_reaches_it() {
    let (mut tk, _outer, inner) = nested_frames();
    let hits = std::rc::Rc::new(std::cell::RefCell::new(Vec::<PathBuf>::new()));
    let sink = hits.clone();
    assert!(tk.bind_file_drop(inner, move |_tk, paths| {
        sink.borrow_mut().extend(paths.iter().cloned());
    }));
    let (x, y) = centre(&mut tk, inner);
    let path = PathBuf::from("/tmp/a.txt");
    assert!(tk.sim_file_drop(x, y, std::slice::from_ref(&path)));
    assert_eq!(*hits.borrow(), vec![path]);
}

/// A drop on an unbound child bubbles up to the bound parent frame.
#[test]
fn drop_on_unbound_child_bubbles_to_parent() {
    let (mut tk, outer, inner) = nested_frames();
    let hits = std::rc::Rc::new(std::cell::RefCell::new(0usize));
    let sink = hits.clone();
    assert!(tk.bind_file_drop(outer, move |_tk, _paths| {
        *sink.borrow_mut() += 1;
    }));
    let (x, y) = centre(&mut tk, inner);
    assert!(tk.sim_file_drop(x, y, &[PathBuf::from("/tmp/b.txt")]));
    assert_eq!(*hits.borrow(), 1);
}

/// With no binding anywhere, a drop returns false.
#[test]
fn drop_without_binding_returns_false() {
    let (mut tk, _outer, inner) = nested_frames();
    let (x, y) = centre(&mut tk, inner);
    assert!(!tk.sim_file_drop(x, y, &[PathBuf::from("/tmp/c.txt")]));
}

/// Hovering between two bound frames fires Leave then Enter, and cancel fires Leave.
#[test]
fn hover_enter_leave_sequence() {
    let mut tk = Tk::new();
    let root = tk.root();
    let left = Frame::new(&mut tk, root).expect("left frame");
    tk.pack(
        left,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    let right = Frame::new(&mut tk, root).expect("right frame");
    tk.pack(
        right,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    tk.sim_resize(400, 300);
    let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::<(WidgetId, DropPhase)>::new()));
    let l = log.clone();
    assert!(tk.bind_file_hover(left, move |_tk, phase| l.borrow_mut().push((left, phase))));
    let r = log.clone();
    assert!(tk.bind_file_hover(right, move |_tk, phase| r.borrow_mut().push((right, phase))));

    let (lx, ly) = centre(&mut tk, left);
    let (rx, ry) = centre(&mut tk, right);
    tk.sim_file_hover(lx, ly, PathBuf::from("/tmp/one.txt"));
    tk.sim_file_hover(rx, ry, PathBuf::from("/tmp/two.txt"));
    tk.sim_file_hover_cancel();

    let entries = log.borrow().clone();
    assert!(entries.contains(&(left, DropPhase::Enter)));
    assert!(entries.contains(&(left, DropPhase::Leave)));
    assert!(entries.contains(&(right, DropPhase::Enter)));
    assert!(entries.contains(&(right, DropPhase::Leave)));
    assert_eq!(entries.last().copied(), Some((right, DropPhase::Leave)));
}

/// The Drop phase fires on the hover callback of the drop target.
#[test]
fn drop_phase_fires_on_hover_callback() {
    let (mut tk, _outer, inner) = nested_frames();
    let phases = std::rc::Rc::new(std::cell::RefCell::new(Vec::<DropPhase>::new()));
    let p = phases.clone();
    assert!(tk.bind_file_hover(inner, move |_tk, phase| p.borrow_mut().push(phase)));
    let (x, y) = centre(&mut tk, inner);
    assert!(tk.sim_file_drop(x, y, &[PathBuf::from("/tmp/d.txt")]));
    assert!(phases.borrow().contains(&DropPhase::Drop));
}

/// A callback that unbinds itself during the drop does not panic, and the
/// second drop returns false.
#[test]
fn callback_can_unbind_itself() {
    let (mut tk, _outer, inner) = nested_frames();
    assert!(tk.bind_file_drop(inner, move |tk, _paths| {
        tk.unbind_file_drop(inner);
    }));
    let (x, y) = centre(&mut tk, inner);
    assert!(tk.sim_file_drop(x, y, &[PathBuf::from("/tmp/e.txt")]));
    assert!(!tk.sim_file_drop(x, y, &[PathBuf::from("/tmp/f.txt")]));
}

/// A binding on a destroyed widget is ignored.
#[test]
fn destroyed_widget_binding_is_ignored() {
    let (mut tk, _outer, inner) = nested_frames();
    let hits = std::rc::Rc::new(std::cell::RefCell::new(0usize));
    let sink = hits.clone();
    assert!(tk.bind_file_drop(inner, move |_tk, _paths| {
        *sink.borrow_mut() += 1;
    }));
    let (x, y) = centre(&mut tk, inner);
    assert!(tk.destroy(inner));
    assert!(!tk.sim_file_drop(x, y, &[PathBuf::from("/tmp/g.txt")]));
    assert_eq!(*hits.borrow(), 0);
}
