use rtk::Tk;

#[test]
fn string_and_int_get_set() {
    let mut tk = Tk::new();
    let s = tk.string_var("x");
    let n = tk.int_var(0);
    assert_eq!(s.get(&tk).as_deref(), Some("x"));
    assert!(s.set(&mut tk, "changed".to_string()));
    assert!(n.set(&mut tk, 42));
    assert_eq!(s.get(&tk).as_deref(), Some("changed"));
    assert_eq!(n.get(&tk), Some(42));
}

#[test]
fn equal_set_does_not_dirty() {
    let mut tk = Tk::new();
    let n = tk.int_var(5);
    let _ = tk.render_to_pixmap(50, 50);
    assert!(!tk.is_dirty());
    n.set(&mut tk, 5);
    assert!(!tk.is_dirty());
    n.set(&mut tk, 6);
    assert!(tk.is_dirty());
}

#[test]
fn trace_that_sets_itself_terminates() {
    let mut tk = Tk::new();
    let n = tk.int_var(0);
    n.trace(&mut tk, move |tk| {
        let v = n.get(tk).unwrap_or(0);
        n.set(tk, v + 1);
    });
    n.set(&mut tk, 1);
    assert_eq!(n.get(&tk), Some(2));
}

#[test]
fn destroyed_var_is_rejected() {
    let mut tk = Tk::new();
    let b = tk.bool_var(true);
    assert!(b.destroy(&mut tk));
    assert_eq!(b.get(&tk), None);
    assert!(!b.set(&mut tk, false));
    assert!(!b.destroy(&mut tk));
}
