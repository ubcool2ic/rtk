/// Handle identifying a scheduled `after`/`every` timer, used to cancel it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimerId(pub(crate) u64);

pub(crate) type TimerCallback = Box<dyn FnMut(&mut crate::Tk)>;

pub(crate) struct Timer {
    pub(crate) id: TimerId,
    pub(crate) due_ms: u64,
    pub(crate) period_ms: Option<u64>,
    pub(crate) callback: Option<TimerCallback>,
}

#[derive(Default)]
pub(crate) struct Timers {
    pub(crate) list: Vec<Timer>,
    pub(crate) next_id: u64,
    pub(crate) now_ms: u64,
    /// Set by `mainloop`: timer clock follows wall time (ms since first `Tk::new`).
    pub(crate) real_clock: bool,
}

impl Timers {
    pub(crate) fn add(
        &mut self,
        delay_ms: u64,
        period_ms: Option<u64>,
        cb: TimerCallback,
    ) -> TimerId {
        if self.real_clock {
            let start = *crate::app::START.get_or_init(std::time::Instant::now);
            self.now_ms = self.now_ms.max(start.elapsed().as_millis() as u64);
        }
        self.next_id = self.next_id.wrapping_add(1);
        let id = TimerId(self.next_id);
        let period_ms = period_ms.map(|p| p.max(1));
        let due_ms = self.now_ms.saturating_add(delay_ms);
        self.list.push(Timer {
            id,
            due_ms,
            period_ms,
            callback: Some(cb),
        });
        id
    }

    pub(crate) fn cancel(&mut self, id: TimerId) -> bool {
        if let Some(pos) = self.list.iter().position(|t| t.id == id) {
            self.list.remove(pos);
            true
        } else {
            false
        }
    }

    pub(crate) fn next_due(&self) -> Option<u64> {
        self.list.iter().map(|t| t.due_ms).min()
    }

    pub(crate) fn len(&self) -> usize {
        self.list.len()
    }
}

impl crate::Tk {
    /// Runs `callback` once, `ms` milliseconds from now (tkinter `after`). Returns a cancel handle.
    pub fn after(
        &mut self,
        ms: u64,
        callback: impl FnMut(&mut crate::Tk) + 'static,
    ) -> TimerId {
        self.timers.add(ms, None, Box::new(callback))
    }

    /// Runs `callback` every `ms` milliseconds (first run after `ms`). Missed ticks are skipped, not queued.
    pub fn every(
        &mut self,
        ms: u64,
        callback: impl FnMut(&mut crate::Tk) + 'static,
    ) -> TimerId {
        self.timers.add(ms, Some(ms), Box::new(callback))
    }

    /// Cancels a pending `after`/`every` timer. Returns false if it already ran or was cancelled.
    pub fn after_cancel(&mut self, id: TimerId) -> bool {
        self.timers.cancel(id)
    }

    /// Number of timers currently scheduled (repeating timers count once).
    pub fn pending_timers(&self) -> usize {
        self.timers.len()
    }

    /// Headless testing: advances the virtual timer clock by `ms` and runs due callbacks.
    pub fn sim_advance_timers(&mut self, ms: u64) -> usize {
        let target_ms = self.timers.now_ms.saturating_add(ms);
        self.run_timers_until(target_ms)
    }

    pub(crate) fn run_timers_until(&mut self, now_ms: u64) -> usize {
        let mut count = 0;
        while count < 10_000 {
            let candidate = self
                .timers
                .list
                .iter()
                .enumerate()
                .filter(|(_, t)| t.due_ms <= now_ms && t.callback.is_some())
                .min_by_key(|(_, t)| (t.due_ms, t.id))
                .map(|(idx, t)| (idx, t.id, t.due_ms, t.period_ms));

            let (idx, id, due_ms, period_ms) = match candidate {
                Some(c) => c,
                None => break,
            };

            self.timers.now_ms = self.timers.now_ms.max(due_ms);

            let mut cb = if let Some(period) = period_ms {
                let cb = self.timers.list[idx].callback.take().unwrap();
                let next_due = due_ms.saturating_add(period);
                if next_due.saturating_add(period) <= now_ms {
                    self.timers.list[idx].due_ms = now_ms.saturating_add(period);
                } else {
                    self.timers.list[idx].due_ms = next_due;
                }
                cb
            } else {
                let timer = self.timers.list.remove(idx);
                timer.callback.unwrap()
            };

            cb(self);

            if period_ms.is_some() {
                if let Some(timer) = self.timers.list.iter_mut().find(|t| t.id == id) {
                    timer.callback = Some(cb);
                }
            }

            self.dirty = true;
            count += 1;
        }

        self.timers.now_ms = self.timers.now_ms.max(now_ms);
        count
    }

    pub(crate) fn next_timer_due_ms(&self) -> Option<u64> {
        self.timers.next_due()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[test]
    fn test_one_shot_fires_once_at_right_time() {
        let mut tk = crate::Tk::new();
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();

        tk.after(50, move |_| {
            c.set(c.get() + 1);
        });

        assert_eq!(tk.sim_advance_timers(30), 0);
        assert_eq!(counter.get(), 0);

        assert_eq!(tk.sim_advance_timers(20), 1);
        assert_eq!(counter.get(), 1);

        assert_eq!(tk.sim_advance_timers(100), 0);
        assert_eq!(counter.get(), 1);
        assert_eq!(tk.pending_timers(), 0);
    }

    #[test]
    fn test_every_repeats() {
        let mut tk = crate::Tk::new();
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();

        tk.every(25, move |_| {
            c.set(c.get() + 1);
        });

        assert_eq!(tk.sim_advance_timers(25), 1);
        assert_eq!(counter.get(), 1);

        assert_eq!(tk.sim_advance_timers(50), 2);
        assert_eq!(counter.get(), 3);
        assert_eq!(tk.pending_timers(), 1);
    }

    #[test]
    fn test_cancel_prevents() {
        let mut tk = crate::Tk::new();
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();

        let id = tk.after(40, move |_| {
            c.set(c.get() + 1);
        });

        assert_eq!(tk.pending_timers(), 1);
        assert!(tk.after_cancel(id));
        assert_eq!(tk.pending_timers(), 0);

        assert_eq!(tk.sim_advance_timers(100), 0);
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_cancel_self_from_inside_callback() {
        let mut tk = crate::Tk::new();
        let counter = Rc::new(Cell::new(0));
        let c = counter.clone();
        let self_id = Rc::new(Cell::new(None));
        let sid = self_id.clone();

        let id = tk.every(20, move |tk| {
            c.set(c.get() + 1);
            if let Some(id) = sid.get() {
                assert!(tk.after_cancel(id));
            }
        });
        self_id.set(Some(id));

        assert_eq!(tk.sim_advance_timers(20), 1);
        assert_eq!(counter.get(), 1);
        assert_eq!(tk.pending_timers(), 0);

        assert_eq!(tk.sim_advance_timers(60), 0);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_scheduling_new_timer_inside_callback() {
        let mut tk = crate::Tk::new();
        let inner_fired = Rc::new(Cell::new(false));
        let inf = inner_fired.clone();

        tk.after(30, move |tk| {
            let inf2 = inf.clone();
            tk.after(20, move |_| {
                inf2.set(true);
            });
        });

        assert_eq!(tk.sim_advance_timers(30), 1);
        assert!(!inner_fired.get());
        assert_eq!(tk.pending_timers(), 1);

        assert_eq!(tk.sim_advance_timers(20), 1);
        assert!(inner_fired.get());
        assert_eq!(tk.pending_timers(), 0);
    }

    #[test]
    fn test_catch_up_skipping() {
        let mut tk = crate::Tk::new();
        let count = Rc::new(Cell::new(0));
        let c = count.clone();

        tk.every(10, move |_| {
            c.set(c.get() + 1);
        });

        // Time advanced far past multiple ticks (from 0 to 100)
        let executed = tk.sim_advance_timers(100);
        assert_eq!(executed, 1);
        assert_eq!(count.get(), 1);
        // Skipped missed ticks and next due is now + period = 100 + 10 = 110
        assert_eq!(tk.next_timer_due_ms(), Some(110));

        assert_eq!(tk.sim_advance_timers(10), 1);
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn test_ordering_by_due_time() {
        let mut tk = crate::Tk::new();
        let order = Rc::new(RefCell::new(Vec::new()));

        let o1 = order.clone();
        tk.after(30, move |_| o1.borrow_mut().push(3));

        let o2 = order.clone();
        tk.after(10, move |_| o2.borrow_mut().push(1));

        let o3 = order.clone();
        tk.after(20, move |_| o3.borrow_mut().push(2));

        let o4 = order.clone();
        tk.after(10, move |_| o4.borrow_mut().push(11));

        assert_eq!(tk.sim_advance_timers(50), 4);
        assert_eq!(*order.borrow(), vec![1, 11, 2, 3]);
    }
}