pub const TAB_WIDTH: usize = 8;

pub fn tabulate(pos: usize, n: usize) -> usize {
    pos + (TAB_WIDTH * n - pos % TAB_WIDTH)
}

pub fn tabulate_back(pos: usize, n: usize) -> usize {
    pos.saturating_sub(pos % TAB_WIDTH + TAB_WIDTH * (n - 1))
}
