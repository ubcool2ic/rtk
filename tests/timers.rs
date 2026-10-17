use rtk::{Label, Opt, Tk};
use std::{cell::Cell, rc::Rc};

#[test]
fn timer_updates_widget_text_and_cancel_handle_works() {
    let mut tk = Tk::new();
    let root = tk.root();
    let l = Label::new(&mut tk, root, "0").unwrap();
    let n = Rc::new(Cell::new(0));
    let c = n.clone();
    let id = tk.every(250, move |tk| { c.set(c.get() + 1); tk.config(l, Opt::Text(c.get().to_string())); });
    let fired: usize = (0..4).map(|_| tk.sim_advance_timers(250)).sum();
    assert_eq!(fired, 4);
    assert_eq!(tk.widget_text(l).as_deref(), Some("4"));
    assert!(tk.after_cancel(id));
    assert_eq!(tk.sim_advance_timers(1000), 0);
    assert_eq!(tk.pending_timers(), 0);
}

#[test]
fn quit_from_timer() {
    let mut tk = Tk::new();
    tk.after(10, |tk| tk.quit());
    tk.sim_advance_timers(10);
    assert!(tk.is_quit_requested());
}
