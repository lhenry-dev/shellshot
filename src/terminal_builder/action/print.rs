use termwiz::surface::{SequenceNo, Surface, SEQ_ZERO};

pub fn process_print(surface: &mut Surface, ch: char) -> SequenceNo {
    surface.add_change(ch);

    SEQ_ZERO
}

pub fn process_print_string(surface: &mut Surface, str: String) -> SequenceNo {
    surface.add_change(str);

    SEQ_ZERO
}
