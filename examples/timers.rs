//! Timer API demo: a ticking clock (`every`), a one-shot message (`after`), and a Stop button (`after_cancel`).
use rtk::{Button, Label, PackOpts, Tk};

/// Runs a timers example with a repeating tick counter and a one-shot delayed message.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("Timers");
    tk.geometry(320, 160);
    let root = tk.root();
    let ticks = Label::new(&mut tk, root, "0 s")?;
    let note = Label::new(&mut tk, root, "waiting...")?;
    let stop = Button::new(&mut tk, root, "Stop")?;
    for w in [ticks, note, stop] { tk.pack(w, PackOpts::default()); }
    let mut n = 0;
    let id = tk.every(1000, move |tk| { n += 1; tk.config(ticks, rtk::Opt::Text(format!("{n} s"))); });
    tk.after(3000, move |tk| { tk.config(note, rtk::Opt::Text("3 seconds passed (after)".into())); });
    tk.set_command(stop, move |tk| { tk.after_cancel(id); });
    tk.mainloop()
}
