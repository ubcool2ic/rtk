//! Pure geometry: no windows, fonts, or toolkit state.
//!
//! Padding is external. Pack consumes a cavity in insertion order. Grid first
//! satisfies individual tracks, then spanning constraints, then applies weights.

use std::ops::{BitOr, BitOrAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
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
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    pub fn inset(self, x: f32, y: f32) -> Self {
        let x = x.max(0.0).min(self.width / 2.0);
        let y = y.max(0.0).min(self.height / 2.0);
        Self::new(
            self.x + x,
            self.y + y,
            self.width - 2.0 * x,
            self.height - 2.0 * y,
        )
    }

    pub fn intersection(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self::new(
            x,
            y,
            (self.x + self.width).min(other.x + other.width) - x,
            (self.y + self.height).min(other.y + other.height) - y,
        )
    }

    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width
            && y < self.y + self.height
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
        size.width.max(0.0) + 2.0 * x.max(0.0),
        size.height.max(0.0) + 2.0 * y.max(0.0),
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
            height = height.max(used_h + size.height);
            used_w += size.width;
        } else {
            width = width.max(used_w + size.width);
            used_h += size.height;
        }
    }
    Size::new(width.max(used_w), height.max(used_h))
}

// Like Tk's XExpansion/YExpansion: orthogonal slaves impose constraints on
// expansion too, so an expanding top slave cannot starve a later left slave.
fn expansion(items: &[PackItem], axis_x: bool, mut available: f32) -> f32 {
    let mut count = 0;
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
    limit.max(0.0)
}

pub fn pack(mut cavity: Rect, items: &[PackItem]) -> Vec<Rect> {
    let mut result = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let opts = item.opts;
        let size = padded(item.requested, opts.padx, opts.pady);
        let axis_x = horizontal(opts.side);
        let available = if axis_x { cavity.width } else { cavity.height };
        let needed = if axis_x { size.width } else { size.height };
        let extra = if opts.expand {
            expansion(&items[index..], axis_x, available)
        } else {
            0.0
        };
        let amount = (needed + extra).min(available);
        let parcel = match opts.side {
            Side::Top => {
                let parcel = Rect::new(cavity.x, cavity.y, cavity.width, amount);
                cavity.y += amount;
                cavity.height -= amount;
                parcel
            }
            Side::Bottom => {
                cavity.height -= amount;
                Rect::new(cavity.x, cavity.y + cavity.height, cavity.width, amount)
            }
            Side::Left => {
                let parcel = Rect::new(cavity.x, cavity.y, amount, cavity.height);
                cavity.x += amount;
                cavity.width -= amount;
                parcel
            }
            Side::Right => {
                cavity.width -= amount;
                Rect::new(cavity.x + cavity.width, cavity.y, amount, cavity.height)
            }
        };
        let inner = parcel.inset(opts.padx, opts.pady);
        let width = if matches!(opts.fill, Fill::X | Fill::Both) {
            inner.width
        } else {
            item.requested.width.max(0.0).min(inner.width)
        };
        let height = if matches!(opts.fill, Fill::Y | Fill::Both) {
            inner.height
        } else {
            item.requested.height.max(0.0).min(inner.height)
        };
        let (ax, ay) = opts.anchor.fractions();
        result.push(Rect::new(
            inner.x + (inner.width - width) * ax,
            inner.y + (inner.height - height) * ay,
            width,
            height,
        ));
    }
    result
}

fn tracks(items: &[GridItem], axis_x: bool) -> Vec<f32> {
    let specifications: Vec<_> = items
        .iter()
        .map(|item| {
            let opts = item.opts;
            let size = padded(item.requested, opts.padx, opts.pady);
            if axis_x {
                (opts.column, opts.columnspan.max(1), size.width)
            } else {
                (opts.row, opts.rowspan.max(1), size.height)
            }
        })
        .collect();
    let count = specifications
        .iter()
        .map(|&(start, span, _)| start + span)
        .max()
        .unwrap_or(0);
    let mut tracks = vec![0.0_f32; count];
    for &(start, span, needed) in &specifications {
        if span == 1 {
            tracks[start] = tracks[start].max(needed);
        }
    }
    for &(start, span, needed) in &specifications {
        if span > 1 {
            let range = &mut tracks[start..start + span];
            let deficit = (needed - range.iter().sum::<f32>()).max(0.0);
            for track in range {
                *track += deficit / span as f32;
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
    if let Some(last) = weights.iter().map(|&(index, _)| index).max() {
        tracks.resize(tracks.len().max(last + 1), 0.0);
    }
    let extra = (available - tracks.iter().sum::<f32>()).max(0.0);
    let total: f32 = weights.iter().map(|&(_, weight)| weight as f32).sum();
    if total > 0.0 {
        for &(index, weight) in weights {
            tracks[index] += extra * weight as f32 / total;
        }
    }
    tracks
}

fn sticky_axis(start: f32, available: f32, requested: f32, low: bool, high: bool) -> (f32, f32) {
    let size = if low && high {
        available
    } else {
        requested.max(0.0).min(available)
    };
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
    let columns = weighted(tracks(items, true), container.width, column_weights);
    let rows = weighted(tracks(items, false), container.height, row_weights);
    items
        .iter()
        .map(|item| {
            let opts = item.opts;
            let cell = Rect::new(
                container.x + columns[..opts.column].iter().sum::<f32>(),
                container.y + rows[..opts.row].iter().sum::<f32>(),
                columns[opts.column..opts.column + opts.columnspan.max(1)]
                    .iter()
                    .sum(),
                rows[opts.row..opts.row + opts.rowspan.max(1)]
                    .iter()
                    .sum(),
            )
            .inset(opts.padx, opts.pady);
            let (x, width) = sticky_axis(
                cell.x,
                cell.width,
                item.requested.width,
                opts.sticky.contains(Sticky::W),
                opts.sticky.contains(Sticky::E),
            );
            let (y, height) = sticky_axis(
                cell.y,
                cell.height,
                item.requested.height,
                opts.sticky.contains(Sticky::N),
                opts.sticky.contains(Sticky::S),
            );
            Rect::new(x, y, width, height)
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
