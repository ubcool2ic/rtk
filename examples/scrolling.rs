//! Scrollable Canvas with a linked Scrollbar: wheel/touchpad, drag the thumb, click to focus then PgUp/PgDn/Home/End.
use rtk::{Canvas, Color, Fill, PackOpts, Scrollbar, Side, Tk};

/// Runs a scrolling example with a canvas of rounded rectangles and a vertical scrollbar.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("Scrolling");
    tk.geometry(420, 320);
    let root = tk.root();
    let c = Canvas::new(&mut tk, root, 400.0, 320.0)?;
    let sb = Scrollbar::new(&mut tk, root)?;
    tk.pack(c, PackOpts { side: Side::Left, ..Default::default() });
    tk.pack(sb, PackOpts { side: Side::Left, fill: Fill::Y, ..Default::default() });
    for i in 0..40 {
        let y = 10.0 + i as f32 * 60.0;
        tk.canvas_create_round_rect(c, 10.0, y, 390.0, y + 50.0, 10.0, Color::rgb(245, 245, 245));
        tk.canvas_create_text_ex(c, 24.0, y + 14.0, format!("Card {i}"), Color::rgb(58, 58, 58), 16.0, 700);
    }
    tk.canvas_scrollregion(c, 400.0, 10.0 + 40.0 * 60.0);
    tk.link_scrollbar(sb, c);
    tk.mainloop()
}
