use std::marker::PhantomData;

use crate::Tk;

/// A Copy, typed index into the variable store of its originating `Tk`.
///
/// Getting a value clones it; setting it invalidates the UI.
#[derive(Debug)]
pub struct Var<T> {
    index: usize,
    marker: PhantomData<fn() -> T>,
}

// Manual implementations deliberately do not require T: Copy or T: Clone.
impl<T> Copy for Var<T> {}

impl<T> Clone for Var<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Var<T> {
    pub fn get(self, tk: &Tk) -> T
    where
        T: Clone,
    {
        tk.vars[self.index]
            .downcast_ref::<T>()
            .expect("variable belongs to a different Tk or has an invalid type")
            .clone()
    }

    pub fn set(self, tk: &mut Tk, value: T) {
        *tk.vars[self.index]
            .downcast_mut::<T>()
            .expect("variable belongs to a different Tk or has an invalid type") = value;
        tk.dirty = true;
    }
}

impl Tk {
    fn make_var<T: 'static>(&mut self, value: T) -> Var<T> {
        let index = self.vars.len();
        self.vars.push(Box::new(value));
        self.dirty = true;
        Var { index, marker: PhantomData }
    }

    pub fn string_var(&mut self, value: impl Into<String>) -> Var<String> {
        self.make_var(value.into())
    }

    pub fn int_var(&mut self, value: i64) -> Var<i64> {
        self.make_var(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Entry, Label, Opt, Tk};

    #[test]
    fn string_and_int_get_set() {
        let mut tk = Tk::new();
        let text = tk.string_var("x");
        let number = tk.int_var(0);
        let copied_text = text;
        assert_eq!(text.get(&tk), "x");
        assert_eq!(number.get(&tk), 0);
        copied_text.set(&mut tk, "changed".into());
        number.set(&mut tk, 42);
        assert_eq!(text.get(&tk), "changed");
        assert_eq!(number.get(&tk), 42);
    }

    #[test]
    fn setting_a_var_marks_dirty() {
        let mut tk = Tk::new();
        let number = tk.int_var(0);
        tk.dirty = false;
        assert!(!tk.is_dirty());
        number.set(&mut tk, 1);
        assert!(tk.is_dirty());
    }

    #[test]
    fn bindings_are_live_and_entries_write_back() {
        let mut tk = Tk::new();
        let root = tk.root();
        let text = tk.string_var("initial");
        let integer = tk.int_var(7);
        let entry = Entry::new(&mut tk, root);
        let label = Label::new(&mut tk, root, "");
        tk.config(entry, Opt::TextVariable(text));
        tk.config(label, Opt::TextVariableInt(integer));
        assert_eq!(tk.entry_get(entry), "initial");
        assert_eq!(tk.display_text(label), "7");
        tk.entry_set(entry, "edited");
        assert_eq!(text.get(&tk), "edited");
        text.set(&mut tk, "external".into());
        assert_eq!(tk.entry_get(entry), "external");
    }
}
