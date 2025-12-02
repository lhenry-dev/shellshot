use termwiz::surface::{SequenceNo, Surface};

pub fn process_print(surface: &mut Surface, ch: char) -> SequenceNo {
    surface.add_change(ch)
}

pub fn process_print_string(surface: &mut Surface, str: &str) -> SequenceNo {
    surface.add_change(str)
}
