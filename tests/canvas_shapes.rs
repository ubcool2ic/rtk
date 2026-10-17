use rtk::{Canvas, Color, PackOpts, Tk};

fn px(tk: &mut Tk, x: u32, y: u32) -> [u8; 4] {
    let p = tk.render_to_pixmap(200, 120).unwrap();
    let i = ((y * 200 + x) * 4) as usize;
    let d = p.data();
    [d[i], d[i + 1], d[i + 2], d[i + 3]]
}

fn setup() -> (Tk, rtk::WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let c = Canvas::new(&mut tk, root, 200.0, 120.0).unwrap();
    tk.pack(c, PackOpts::default());
    tk.sim_resize(200, 120);
    (tk, c)
}

#[test]
fn round_rect_fills_center_but_not_corner() {
    let (mut tk, c) = setup();
    let r = tk.canvas_create_round_rect(c, 10.0, 10.0, 110.0, 70.0, 16.0, Color::rgb(242, 106, 27)).unwrap();
    let rect = tk.widget_rect(c).unwrap();
    let (ox, oy) = (rect.x as u32, rect.y as u32);
    assert_eq!(&px(&mut tk, ox + 60, oy + 40)[..3], &[242, 106, 27]);
    assert_ne!(&px(&mut tk, ox + 11, oy + 11)[..3], &[242, 106, 27], "corner should be rounded off");
    assert_eq!(tk.canvas_coords(c, r), Some([10.0, 10.0, 110.0, 70.0]));
}

#[test]
fn polyline_polygon_ring_text_ex_render_and_move() {
    let (mut tk, c) = setup();
    assert!(tk.canvas_create_polyline(c, &[(0.0, 0.0)], Color::rgb(0, 0, 0), 2.0).is_none());
    let l = tk.canvas_create_polyline(c, &[(10.0, 100.0), (50.0, 20.0), (90.0, 80.0)], Color::rgb(0, 0, 255), 3.0).unwrap();
    assert_eq!(tk.canvas_coords(c, l), Some([10.0, 20.0, 90.0, 100.0]));
    tk.canvas_move(c, l, 5.0, 0.0);
    assert_eq!(tk.canvas_coords(c, l), Some([15.0, 20.0, 95.0, 100.0]));
    let poly = tk.canvas_create_polygon(c, &[(120.0, 10.0), (190.0, 10.0), (155.0, 60.0)], Color::rgb(0, 160, 0)).unwrap();
    tk.canvas_create_ring(c, 120.0, 70.0, 160.0, 110.0, Color::rgb(200, 0, 0), 3.0).unwrap();
    tk.canvas_create_text_ex(c, 10.0, 10.0, "42", Color::rgb(30, 30, 30), 28.0, 700).unwrap();
    assert_eq!(tk.canvas_items(c), 4);
    let rect = tk.widget_rect(c).unwrap();
    let (ox, oy) = (rect.x as u32, rect.y as u32);
    assert_eq!(&px(&mut tk, ox + 155, oy + 25)[..3], &[0, 160, 0]);
    let _ = poly;
    let (w_big, h_big) = tk.text_size("42", 28.0, 700);
    let (w_small, _) = tk.text_size("42", 12.0, 400);
    assert!(w_big > w_small * 1.8 && h_big > 28.0);
}
