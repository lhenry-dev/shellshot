use termwiz::surface::Surface;

use crate::terminal_builder::constants::TAB_WIDTH;

#[expect(dead_code)]
pub fn ensure_height(surface: &mut Surface, min_height: usize) {
    let (width, height) = surface.dimensions();

    if min_height >= height {
        surface.resize(width, min_height + 1);
    }
}

pub fn tabulate(pos: usize, n: usize) -> usize {
    pos + (TAB_WIDTH * n - pos % TAB_WIDTH)
}

pub fn tabulate_back(pos: usize, n: usize) -> usize {
    pos.saturating_sub(pos % TAB_WIDTH + TAB_WIDTH * (n - 1))
}
