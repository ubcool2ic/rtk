//! Pure, bounded geometry, independent of fonts and windows.
//!
//! Nonfinite dimensions and padding become zero. Grid indices are capped at
//! 4095, spans at the remaining track count, and undersized grids shrink.
//! Results are contained in the normalized parent rectangle.

use std::ops::{BitOr, BitOrAssign};

pub(crate) const MAX_TRACKS: usize = 4096;

pub(crate) fn length(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 10_000_000.0)
    } else {
        0.0
    }
}

fn position(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1_000_000_000.0, 1_000_000_000.0)
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width: length(width), height: length(height) }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x: position(x),
            y: position(y),
            width: length(width),
            height: length(height),
        }
    }

    fn normalized(self) -> Self {
        Self::new(self.x, self.y, self.width, self.height)
    }

    pub fn inset(self, x: f32, y: f32) -> Self {
        let rect = self.normalized();
        let x = length(x).min(rect.width / 2.0);
        let y = length(y).min(rect.height / 2.0);
        Self::new(
            rect.x + x,
            rect.y + y,
            rect.width - 2.0 * x,
            rect.height - 2.0 * y,
        )
    }

    pub fn intersection(self, other: Self) -> Self {
        let a = self.normalized();
        let b = other.normalized();
        let x = a.x.max(b.x);
        let y = a.y.max(b.y);
        Self::new(
            x,
            y,
            (a.x + a.width).min(b.x + b.width) - x,
            (a.y + a.height).min(b.y + b.height) - y,
        )
    }

    /// Clamps even empty rectangles' origins into the parent.
    pub fn within(self, parent: Self) -> Self {
        let parent = parent.normalized();
        let rect = self.normalized();
        let right = parent.x + parent.width;
        let bottom = parent.y + parent.height;
        let x = rect.x.clamp(parent.x, right);
        let y = rect.y.clamp(parent.y, bottom);
        Self::new(
            x,
            y,
            ((rect.x + rect.width).min(right) - x).max(0.0),
            ((rect.y + rect.height).min(bottom) - y).max(0.0),
        )
    }

    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width
            && y < self.y + self.height
    }

    pub(crate) fn scaled(self, scale: f32) -> Self {
        Self::new(
            self.x * scale,
            self.y * scale,
            self.width * scale,
            self.height * scale,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fill {
    #[default]
    None,
    X,
    Y,
    Both,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Anchor {
    #[default]
    Center,
    N,
    S,
    E,
    W,
    Ne,
    Nw,
    Se,
    Sw,
}

impl Anchor {
    fn fractions(self) -> (f32, f32) {
        match self {
            Self::Center => (0.5, 0.5),
            Self::N => (0.5, 0.0),
            Self::S => (0.5, 1.0),
            Self::E => (1.0, 0.5),
            Self::W => (0.0, 0.5),
            Self::Ne => (1.0, 0.0),
            Self::Nw => (0.0, 0.0),
            Self::Se => (1.0, 1.0),
            Self::Sw => (0.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PackOpts {
    pub side: Side,
    pub fill: Fill,
    pub expand: bool,
    pub padx: f32,
    pub pady: f32,
    pub anchor: Anchor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sticky(u8);

impl Sticky {
    pub const NONE: Self = Self(0);
    pub const N: Self = Self(1);
    pub const S: Self = Self(2);
    pub const E: Self = Self(4);
    pub const W: Self = Self(8);
    pub const NS: Self = Self(3);
    pub const EW: Self = Self(12);
    pub const NSEW: Self = Self(15);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for Sticky {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Sticky {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GridOpts {
    pub row: usize,
    pub column: usize,
    pub rowspan: usize,
    pub columnspan: usize,
    pub sticky: Sticky,
    pub padx: f32,
    pub pady: f32,
}

impl Default for GridOpts {
    fn default() -> Self {
        Self {
            row: 0,
            column: 0,
            rowspan: 1,
            columnspan: 1,
            sticky: Sticky::NONE,
            padx: 0.0,
            pady: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PackItem {
    pub requested: Size,
    pub opts: PackOpts,
}

#[derive(Clone, Copy, Debug)]
pub struct GridItem {
    pub requested: Size,
    pub opts: GridOpts,
}

fn padded(size: Size, x: f32, y: f32) -> Size {
    Size::new(
        length(size.width) + 2.0 * length(x),
        length(size.height) + 2.0 * length(y),
    )
}

fn horizontal(side: Side) -> bool {
    matches!(side, Side::Left | Side::Right)
}

pub fn pack_request(items: &[PackItem]) -> Size {
    let (mut used_w, mut used_h) = (0.0_f32, 0.0_f32);
    let (mut width, mut height) = (0.0_f32, 0.0_f32);
    for item in items {
        let size = padded(item.requested, item.opts.padx, item.opts.pady);
        if horizontal(item.opts.side) {
            height = length(height.max(used_h + size.height));
            used_w = length(used_w + size.width);
        } else {
            width = length(width.max(used_w + size.width));
            used_h = length(used_h + size.height);
        }
    }
    Size::new(width.max(used_w), height.max(used_h))
}

fn expansion(items: &[PackItem], axis_x: bool, mut available: f32) -> f32 {
    let mut count = 0_usize;
    let mut limit = available;
    for item in items {
        let size = padded(item.requested, item.opts.padx, item.opts.pady);
        let needed = if axis_x { size.width } else { size.height };
        if horizontal(item.opts.side) == axis_x {
            available -= needed;
            count += usize::from(item.opts.expand);
        } else if count > 0 {
            limit = limit.min((available - needed) / count as f32);
        }
    }
    if count > 0 {
        limit = limit.min(available / count as f32);
    }
    length(limit)
}

pub fn pack(container: Rect, items: &[PackItem]) -> Vec<Rect> {
    let container = container.normalized();
    let mut cavity = container;
    let mut result = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let opts = item.opts;
        let size = padded(item.requested, opts.padx, opts.pady);
        let axis_x = horizontal(opts.side);
        let available = if axis_x { cavity.width } else { cavity.height };
        let needed = if axis_x { size.width } else { size.height };
        let extra = if opts.expand {
            expansion(items.get(index..).unwrap_or_default(), axis_x, available)
        } else {
            0.0
        };
        let amount = (needed + extra).min(available);
        let parcel = match opts.side {
            Side::Top => {
                let parcel = Rect::new(cavity.x, cavity.y, cavity.width, amount);
                cavity.y += amount;
                cavity.height = (cavity.height - amount).max(0.0);
                parcel
            }
            Side::Bottom => {
                cavity.height = (cavity.height - amount).max(0.0);
                Rect::new(cavity.x, cavity.y + cavity.height, cavity.width, amount)
            }
            Side::Left => {
                let parcel = Rect::new(cavity.x, cavity.y, amount, cavity.height);
                cavity.x += amount;
                cavity.width = (cavity.width - amount).max(0.0);
                parcel
            }
            Side::Right => {
                cavity.width = (cavity.width - amount).max(0.0);
                Rect::new(cavity.x + cavity.width, cavity.y, amount, cavity.height)
            }
        };
        let inner = parcel.within(container).inset(opts.padx, opts.pady);
        let width = if matches!(opts.fill, Fill::X | Fill::Both) {
            inner.width
        } else {
            length(item.requested.width).min(inner.width)
        };
        let height = if matches!(opts.fill, Fill::Y | Fill::Both) {
            inner.height
        } else {
            length(item.requested.height).min(inner.height)
        };
        let (ax, ay) = opts.anchor.fractions();
        result.push(
            Rect::new(
                inner.x + (inner.width - width) * ax,
                inner.y + (inner.height - height) * ay,
                width,
                height,
            )
            .within(container),
        );
    }
    result
}

fn span(start: usize, count: usize) -> (usize, usize) {
    let start = start.min(MAX_TRACKS - 1);
    (start, count.max(1).min(MAX_TRACKS - start))
}

fn specification(item: &GridItem, axis_x: bool) -> (usize, usize, f32) {
    let opts = item.opts;
    let size = padded(item.requested, opts.padx, opts.pady);
    let (start, count) = if axis_x {
        span(opts.column, opts.columnspan)
    } else {
        span(opts.row, opts.rowspan)
    };
    (start, count, if axis_x { size.width } else { size.height })
}

fn tracks(items: &[GridItem], axis_x: bool) -> Vec<f32> {
    let count = items
        .iter()
        .map(|item| {
            let (start, count, _) = specification(item, axis_x);
            start + count
        })
        .max()
        .unwrap_or(0);
    let mut tracks = vec![0.0_f32; count];
    for item in items {
        let (start, count, needed) = specification(item, axis_x);
        if count == 1 {
            if let Some(track) = tracks.get_mut(start) {
                *track = track.max(needed);
            }
        }
    }
    for item in items {
        let (start, count, needed) = specification(item, axis_x);
        if count > 1 {
            if let Some(range) = tracks.get_mut(start..start + count) {
                let deficit = (needed - range.iter().sum::<f32>()).max(0.0);
                for track in range {
                    *track += deficit / count as f32;
                }
            }
        }
    }
    tracks
}

pub fn grid_request(items: &[GridItem]) -> Size {
    Size::new(
        tracks(items, true).iter().sum(),
        tracks(items, false).iter().sum(),
    )
}

fn weighted(mut tracks: Vec<f32>, available: f32, weights: &[(usize, u32)]) -> Vec<f32> {
    let last = weights.iter().filter(|(i, _)| *i < MAX_TRACKS).map(|(i, _)| *i).max();
    if let Some(last) = last {
        tracks.resize(tracks.len().max(last + 1), 0.0);
    }
    let requested: f32 = tracks.iter().sum();
    if requested > available && requested > 0.0 {
        let ratio = available / requested;
        for track in &mut tracks {
            *track *= ratio;
        }
    } else {
        let extra = (available - requested).max(0.0);
        let total: f64 = weights
            .iter()
            .filter(|(i, _)| *i < MAX_TRACKS)
            .map(|(_, weight)| f64::from(*weight))
            .sum();
        if total > 0.0 {
            for &(index, weight) in weights {
                if let Some(track) = tracks.get_mut(index) {
                    *track += (f64::from(extra) * f64::from(weight) / total) as f32;
                }
            }
        }
    }
    tracks
}

fn track_range(tracks: &[f32], start: usize, count: usize, available: f32) -> (f32, f32) {
    let offset = tracks.get(..start).unwrap_or_default().iter().sum::<f32>().min(available);
    let size = tracks
        .get(start..start + count)
        .unwrap_or_default()
        .iter()
        .sum::<f32>()
        .min((available - offset).max(0.0));
    (offset, size)
}

fn sticky_axis(start: f32, available: f32, requested: f32, low: bool, high: bool) -> (f32, f32) {
    let size = if low && high { available } else { length(requested).min(available) };
    let offset = if low {
        0.0
    } else if high {
        available - size
    } else {
        (available - size) / 2.0
    };
    (start + offset, size)
}

pub fn grid(
    container: Rect,
    items: &[GridItem],
    column_weights: &[(usize, u32)],
    row_weights: &[(usize, u32)],
) -> Vec<Rect> {
    let container = container.normalized();
    let columns = weighted(tracks(items, true), container.width, column_weights);
    let rows = weighted(tracks(items, false), container.height, row_weights);
    items
        .iter()
        .map(|item| {
            let opts = item.opts;
            let (column, columnspan) = span(opts.column, opts.columnspan);
            let (row, rowspan) = span(opts.row, opts.rowspan);
            let (dx, width) = track_range(&columns, column, columnspan, container.width);
            let (dy, height) = track_range(&rows, row, rowspan, container.height);
            let cell = Rect::new(container.x + dx, container.y + dy, width, height)
                .within(container)
                .inset(opts.padx, opts.pady);
            let (x, width) = sticky_axis(
                cell.x, cell.width, item.requested.width,
                opts.sticky.contains(Sticky::W), opts.sticky.contains(Sticky::E),
            );
            let (y, height) = sticky_axis(
                cell.y, cell.height, item.requested.height,
                opts.sticky.contains(Sticky::N), opts.sticky.contains(Sticky::S),
            );
            Rect::new(x, y, width, height).within(container)
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;

    fn slave(width: f32, height: f32, opts: PackOpts) -> PackItem {
        PackItem { requested: Size::new(width, height), opts }
    }

    fn cell(width: f32, row: usize, column: usize) -> GridItem {
        GridItem {
            requested: Size::new(width, 10.0),
            opts: GridOpts { row, column, ..GridOpts::default() },
        }
    }

    #[test]
    fn pack_top_stacking() {
        let items = [
            slave(20.0, 10.0, PackOpts::default()),
            slave(30.0, 15.0, PackOpts::default()),
        ];
        let result = pack(Rect::new(0.0, 0.0, 100.0, 80.0), &items);
        assert_eq!(result[0], Rect::new(40.0, 0.0, 20.0, 10.0));
        assert_eq!(result[1], Rect::new(35.0, 10.0, 30.0, 15.0));
        assert_eq!(pack_request(&items), Size::new(30.0, 25.0));
    }

    #[test]
    fn pack_left_fill_y() {
        let item = slave(20.0, 10.0, PackOpts {
            side: Side::Left, fill: Fill::Y, ..PackOpts::default()
        });
        assert_eq!(
            pack(Rect::new(0.0, 0.0, 100.0, 80.0), &[item])[0],
            Rect::new(0.0, 0.0, 20.0, 80.0)
        );
    }

    #[test]
    fn pack_expand_sharing() {
        let opts = PackOpts { fill: Fill::Both, expand: true, ..PackOpts::default() };
        let result = pack(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            &[slave(10.0, 10.0, opts), slave(10.0, 10.0, opts)],
        );
        assert_eq!(result[0], Rect::new(0.0, 0.0, 100.0, 50.0));
        assert_eq!(result[1], Rect::new(0.0, 50.0, 100.0, 50.0));
    }

    #[test]
    fn pack_padding() {
        let opts = PackOpts { fill: Fill::X, padx: 3.0, pady: 4.0, ..PackOpts::default() };
        let items = [slave(20.0, 10.0, opts), slave(20.0, 10.0, opts)];
        let result = pack(Rect::new(0.0, 0.0, 100.0, 100.0), &items);
        assert_eq!(result[0], Rect::new(3.0, 4.0, 94.0, 10.0));
        assert_eq!(result[1].y, 22.0);
        assert_eq!(pack_request(&items), Size::new(26.0, 36.0));
    }

    #[test]
    fn expansion_reserves_orthogonal_requests() {
        let items = [
            slave(10.0, 10.0, PackOpts {
                expand: true, fill: Fill::Both, ..PackOpts::default()
            }),
            slave(20.0, 40.0, PackOpts {
                side: Side::Left, fill: Fill::Y, ..PackOpts::default()
            }),
        ];
        let result = pack(Rect::new(0.0, 0.0, 100.0, 80.0), &items);
        assert_eq!(result[0].height, 40.0);
        assert_eq!(result[1].height, 40.0);
    }

    #[test]
    fn grid_column_widths_are_maxima() {
        let items = [cell(10.0, 0, 0), cell(30.0, 1, 0), cell(20.0, 0, 1)];
        assert_eq!(grid_request(&items), Size::new(50.0, 20.0));
        let result = grid(Rect::new(0.0, 0.0, 50.0, 20.0), &items, &[], &[]);
        assert_eq!(result[2].x, 30.0);
    }

    #[test]
    fn grid_sticky_ew_stretches() {
        let mut item = cell(20.0, 0, 0);
        item.opts.sticky = Sticky::E | Sticky::W;
        let result = grid(Rect::new(0.0, 0.0, 100.0, 20.0), &[item], &[(0, 1)], &[]);
        assert_eq!(result[0].width, 100.0);
    }

    #[test]
    fn grid_weight_distribution() {
        let mut items = [cell(10.0, 0, 0), cell(10.0, 0, 1)];
        for item in &mut items {
            item.opts.sticky = Sticky::NSEW;
        }
        let result = grid(
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &items, &[(0, 1), (1, 3)], &[(0, 1)],
        );
        assert_eq!(result[0], Rect::new(0.0, 0.0, 30.0, 30.0));
        assert_eq!(result[1], Rect::new(30.0, 0.0, 70.0, 30.0));
    }

    #[test]
    fn grid_spans_and_padding() {
        let mut item = cell(40.0, 0, 0);
        item.opts.columnspan = 2;
        item.opts.rowspan = 2;
        item.opts.padx = 5.0;
        item.opts.pady = 3.0;
        item.opts.sticky = Sticky::NSEW;
        assert_eq!(grid_request(&[item]), Size::new(50.0, 16.0));
        assert_eq!(
            grid(Rect::new(0.0, 0.0, 50.0, 16.0), &[item], &[], &[])[0],
            Rect::new(5.0, 3.0, 40.0, 10.0)
        );
    }
}
