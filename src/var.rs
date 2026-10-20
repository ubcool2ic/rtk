//! Typed, generational variable handles (tkinter `StringVar`, `IntVar`, ...).
//!
//! Values live inside the owning [`Tk`]; a [`Var`] is a cheap `Copy` handle
//! that becomes invalid once the variable is destroyed.

use std::any::Any;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::{Command, Tk};

/// A typed, generational variable handle.
///
/// `get` returns `None` for a destroyed/foreign handle. Mutations return false
/// for invalid handles. Equal assignments succeed without invalidating the UI.
/// Traces run synchronously, at most once per active notification stack for
/// each variable. Reentrant writes are stored but do not recursively notify.
///
/// Handles are `Send + Sync` (they carry no data, only indices), so they can be
/// moved into `TkProxy::post` closures; all access still goes through `&Tk`.
#[derive(Debug)]
pub struct Var<T> {
    index: usize,
    gen: u32,
    /// Identity token of the owning toolkit: the address of its `var_owner`
    /// allocation, stored as an integer and never dereferenced.
    owner: usize,
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Var<T> {}

impl<T> Clone for Var<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Var<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.gen == other.gen && self.owner == other.owner
    }
}

impl<T> Eq for Var<T> {}

impl<T> Hash for Var<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.gen.hash(state);
        self.owner.hash(state);
    }
}

pub(crate) struct VarSlot {
    gen: u32,
    value: Option<Box<dyn Any>>,
    trace: Option<Command>,
    notifying: bool,
    // Keep the allocation alive even if a variable is destroyed. Its address
    // cannot be reused while this toolkit is alive.
    owner: Arc<()>,
}

impl<T: 'static> Var<T> {
    fn slot(self, tk: &Tk) -> Option<&VarSlot> {
        tk.vars.get(self.index).filter(|slot| {
            slot.gen == self.gen
                && Arc::as_ptr(&slot.owner) as usize == self.owner
                && slot.value.as_ref().is_some_and(|value| value.is::<T>())
        })
    }

    fn slot_mut(self, tk: &mut Tk) -> Option<&mut VarSlot> {
        tk.vars.get_mut(self.index).filter(|slot| {
            slot.gen == self.gen
                && Arc::as_ptr(&slot.owner) as usize == self.owner
                && slot.value.as_ref().is_some_and(|value| value.is::<T>())
        })
    }

    /// Returns a copy of the current value, or `None` if the handle is invalid.
    pub fn get(self, tk: &Tk) -> Option<T>
    where
        T: Clone,
    {
        self.slot(tk)?.value.as_ref()?.downcast_ref::<T>().cloned()
    }

    /// Assigns a new value, running the trace if it changed. Returns false for
    /// an invalid handle.
    pub fn set(self, tk: &mut Tk, value: T) -> bool
    where
        T: PartialEq,
    {
        let Some(slot) = self.slot_mut(tk) else { return false };
        let Some(stored) = slot.value.as_mut().and_then(|value| value.downcast_mut::<T>()) else {
            return false;
        };
        if *stored == value {
            return true;
        }
        *stored = value;
        let notify = !slot.notifying;
        let command = if notify {
            slot.notifying = true;
            slot.trace.take()
        } else {
            None
        };
        // A variable change may affect any widget bound to it; conservatively
        // request a full redraw. Widgets that only repaint their own contents
        // (Entry/Text editing) invalidate themselves separately.
        tk.dirty = true;
        if let Some(mut command) = command {
            command(tk);
            if let Some(slot) = self.slot_mut(tk) {
                if slot.trace.is_none() {
                    slot.trace = Some(command);
                }
            }
        }
        if notify {
            if let Some(slot) = self.slot_mut(tk) {
                slot.notifying = false;
            }
        }
        true
    }

    /// Installs or replaces the single change trace.
    pub fn trace(self, tk: &mut Tk, command: impl FnMut(&mut Tk) + 'static) -> bool {
        let Some(slot) = self.slot_mut(tk) else { return false };
        slot.trace = Some(Box::new(command));
        true
    }

    /// Invalidates this handle and all copies. Bound widgets remain usable.
    pub fn destroy(self, tk: &mut Tk) -> bool {
        let Some(slot) = self.slot_mut(tk) else { return false };
        slot.value = None;
        slot.trace = None;
        slot.notifying = false;
        slot.gen = slot.gen.saturating_add(1);
        tk.dirty = true;
        true
    }
}

impl Tk {
    fn make_var<T: 'static>(&mut self, value: T) -> Var<T> {
        // Variable slots are intentionally not reused. This also makes a
        // destroyed handle unambiguously invalid at generation exhaustion.
        let owner = Arc::clone(&self.var_owner);
        let handle = Var {
            index: self.vars.len(),
            gen: 0,
            owner: Arc::as_ptr(&owner) as usize,
            marker: PhantomData,
        };
        self.vars.push(VarSlot {
            gen: 0,
            value: Some(Box::new(value)),
            trace: None,
            notifying: false,
            owner,
        });
        self.dirty = true;
        handle
    }

    /// Creates a string variable (tkinter `StringVar`).
    pub fn string_var(&mut self, value: impl Into<String>) -> Var<String> {
        self.make_var(value.into())
    }

    /// Creates an integer variable (tkinter `IntVar`).
    pub fn int_var(&mut self, value: i64) -> Var<i64> {
        self.make_var(value)
    }

    /// Creates a boolean variable (tkinter `BooleanVar`).
    pub fn bool_var(&mut self, value: bool) -> Var<bool> {
        self.make_var(value)
    }

    /// Creates a floating-point variable (tkinter `DoubleVar`).
    pub fn f64_var(&mut self, value: f64) -> Var<f64> {
        self.make_var(value)
    }
}
