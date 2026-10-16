# rtk

A first working, tkinter-style GUI library for Rust 1.85 and edition 2021.

`rtk` combines an imperative widget API with Rust-owned application state:
construct widgets, configure them, choose `pack` or `grid`, and run the event
loop. The same renderer also produces PNG screenshots without a window.

## Status

Version 0.1 is a deliberately small desktop prototype, not a Tk binding or
a production-complete toolkit.

Implemented:

- Frame, Label, Button, and single-line Entry.
- Copy widget IDs and typed Copy variable handles.
- String bindings for labels and entries; integer bindings for display.
- Button commands with mutable access to the toolkit.
- Recursive requested-size calculation and resize-driven layout.
- Cavity-based pack with sides, fill, expansion, padding, and anchors.
- Grid with spans, sticky alignment, padding, and weighted rows/columns.
- Software rendering, system-font discovery, advanced text shaping, and
  headless PNG output.
- Entry click positioning, typing, Backspace, Delete, Left, Right, Home,
  End, horizontal scrolling, and a visible focused caret.

Known limitations:

- No selection, clipboard, Tab navigation, keyboard button activation, IME,
  or blinking caret yet.
- Editing moves by Unicode scalar value, not grapheme cluster. Text shaping
  supports complex scripts, but caret placement is an approximate prefix-width
  calculation, not a complete bidirectional-text editor.
- Coordinates and font sizes are physical pixels; automatic HiDPI scaling is
  not implemented.
- Grid distributes positive extra space. When undersized, its requested tracks
  are clipped rather than proportionally shrunk.
- Widget lifetime is the toolkit lifetime; there is no destroy/reparent API.
- Rendering and text measurement favor simplicity over caching/performance.

## tkinter and rtk: hello

<table>
<tr><th>Python / tkinter</th><th>Rust / rtk</th></tr>
<tr><td valign="top"><pre>
import tkinter as tk

root = tk.Tk()
root.title("Hello")

tk.Label(
    root, text="Hello, rtk!"
).pack(pady=10)

tk.Button(
    root,
    text="Quit",
    command=root.destroy,
).pack(pady=10)

root.mainloop()
</pre></td><td valign="top"><pre>
use rtk::{Button, Label, PackOpts, Tk};

let mut tk = Tk::new();
tk.title("Hello");
let root = tk.root();
let padding = PackOpts {
    pady: 10.0,
    ..PackOpts::default()
};

let label = Label::new(
    &amp;mut tk, root, "Hello, rtk!"
);
tk.pack(label, padding);

let quit = Button::new(&amp;mut tk, root, "Quit");
tk.pack(quit, padding);
tk.set_command(quit, |tk| tk.quit());

tk.mainloop()?;
</pre></td></tr>
</table>

The Rust snippets are function bodies; complete programs are in `examples/`.

## tkinter and rtk: counter

<table>
<tr><th>Python / tkinter</th><th>Rust / rtk</th></tr>
<tr><td valign="top"><pre>
import tkinter as tk

root = tk.Tk()
count = tk.IntVar(value=0)
text = tk.StringVar(value="Count: 0")

tk.Label(
    root, textvariable=text
).pack()

def increment():
    n = count.get() + 1
    count.set(n)
    text.set(f"Count: {n}")

tk.Button(
    root, text="+1", command=increment
).pack()

root.mainloop()
</pre></td><td valign="top"><pre>
use rtk::{Button, Label, Opt, PackOpts, Tk};

let mut tk = Tk::new();
let root = tk.root();
let count = tk.int_var(0);
let text = tk.string_var("Count: 0");

let label = Label::new(&amp;mut tk, root, "");
tk.config(label, Opt::TextVariable(text));
tk.pack(label, PackOpts::default());

let button = Button::new(&amp;mut tk, root, "+1");
tk.set_command(button, move |tk| {
    let n = count.get(tk) + 1;
    count.set(tk, n);
    text.set(tk, format!("Count: {n}"));
});
tk.pack(button, PackOpts::default());

tk.mainloop()?;
</pre></td></tr>
</table>

For a plain numeric label, bind the integer directly with
`Opt::TextVariableInt(count)`.

## Building and running

Install Rust 1.85 or newer, a desktop display server for interactive examples,
and at least one system font. On Debian/Ubuntu, useful packages are:

```sh
sudo apt-get install libxkbcommon-dev libwayland-dev \
  libx11-dev libxi-dev libxrandr-dev libxcursor-dev \
  libxinerama-dev fonts-dejavu-core
```

Then:

```sh
cargo build --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings

cargo run --example hello
cargo run --example counter
cargo run --example form
cargo run --example screenshot -- hello.png

RTK_PNG=test-render.png cargo test --test render
```

Headless rendering does not create a winit event loop and does not need
`DISPLAY` or a Wayland connection. It still discovers system fonts through
cosmic-text's fontdb. Tests do not depend on exact glyph pixels.

The dependency requirements intentionally use the requested stable API series:
winit 0.30, softbuffer 0.4, tiny-skia 0.11, and cosmic-text 0.12. This library
does not commit a lockfile. The Cargo resolver configuration prefers transitive
dependency releases compatible with the declared Rust version.

`ci/rust.yml` is a ready-to-use GitHub Actions configuration; move it to
`.github/workflows/rust.yml` to activate it.

## API details

- Store `let root = tk.root()` before passing `&mut tk` to constructors.
- Constructors return `WidgetId`. Widgets become visible after `pack` or `grid`.
- Only frames, including the root, may contain children.
- Do not mix pack and grid among siblings. Nested frames may use different
  geometry managers.
- Pack order is child creation order. Reconfiguring a managed widget preserves
  its position in that order.
- Padding values are pixels. `Opt::Width(n)` specifies a character-based width.
- Grid weights default to zero. Combine sticky flags with `|`, or use
  `Sticky::EW`, `Sticky::NS`, and `Sticky::NSEW`.
- `geometry(w, h)` overrides the initial window size. Without it, the root's
  recursive requested size is used, with a minimum of 200×100.
- `title()` updates the title; `quit()` requests event-loop termination.
- `Var::get` clones its value. `Var::set` marks the UI dirty. Bindings are read
  directly during measurement/rendering; no observer graph is needed.
- An Entry writes edits back to its bound string variable. `entry_get` returns
  its current value, and `entry_set` replaces it. Keep externally assigned
  entry variables single-line.
- Bound text takes precedence over a label's static `Opt::Text`.
- Callback replacement from inside the running callback is supported.
- Invalid IDs, incompatible configuration, mixed geometry managers, and invalid
  image dimensions are programmer errors and may panic.
- IDs and variables belong to their originating `Tk`; never mix handles from
  different toolkit instances.

`render_to_pixmap(w, h)` returns a `tiny_skia::Pixmap`.
`save_png(path, w, h)` returns a `Result` and performs no window operations.
`mainloop()` consumes `Tk`, returns a `Result`, and must run on the main thread.

## Design

`Tk` owns a `Vec<WidgetNode>` arena. Each widget handle is a Copy `WidgetId(u32)`;
each variable handle is an index plus `PhantomData`. The toolkit owns the actual
values, widget tree, callbacks, and renderer.

There is no `Rc<RefCell<_>>` in the application API. Instead of a graph of
individually borrowed widgets, commands receive one `&mut Tk`. They can read
variables, update labels, create widgets, or request shutdown through that
exclusive borrow.

A command is temporarily taken out of its node before invocation, then restored
unless the callback installed a replacement. This prevents borrowing the arena
while calling a closure that itself mutates the arena.

The window backend uses an internal `Arc<Window>` solely to satisfy owned
window/display-handle requirements; it does not expose shared mutable widget
state.

Layout is independent of fonts and windows in `src/layout.rs`. Widget
measurement uses cosmic-text, and rendering targets tiny-skia before copying
premultiplied RGBA pixels into softbuffer's `0x00RRGGBB` format.

## License

MIT OR Apache-2.0. The MIT license text is included in `LICENSE`.
