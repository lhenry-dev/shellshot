use std::io::{self};

use termwiz::{
    escape::{CSI, csi::Edit},
    surface::{Change, Position, SEQ_ZERO, SequenceNo, Surface},
};

mod cursor;
mod sgr;

use crate::terminal_builder::action::csi::{cursor::process_cursor, sgr::process_sgr};

pub fn process_csi(surface: &mut Surface, writer: &mut dyn io::Write, csi: &CSI) -> SequenceNo {
    match csi {
        CSI::Sgr(sgr) => process_sgr(surface, sgr),
        CSI::Cursor(cursor) => process_cursor(surface, writer, cursor),
        CSI::Edit(edit) => process_edit(surface, edit),
        CSI::Mode(_)
        | CSI::Device(_)
        | CSI::Mouse(_)
        | CSI::Window(_)
        | CSI::Keyboard(_)
        | CSI::SelectCharacterPath(_, _)
        | CSI::Unspecified(_) => SEQ_ZERO,
    }
}

fn process_edit(surface: &mut Surface, edit: &Edit) -> SequenceNo {
    match edit {
        Edit::EraseCharacter(n) => {
            let (x, y) = surface.cursor_position();
            surface.add_change(Change::Text(" ".repeat(*n as usize)));
            surface.add_change(Change::CursorPosition {
                x: Position::Absolute(x),
                y: Position::Absolute(y),
            })
        }
        Edit::EraseInLine(_)
        | Edit::InsertCharacter(_)
        | Edit::InsertLine(_)
        | Edit::ScrollDown(_)
        | Edit::ScrollUp(_)
        | Edit::EraseInDisplay(_)
        | Edit::Repeat(_)
        | Edit::DeleteCharacter(_)
        | Edit::DeleteLine(_) => SEQ_ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termwiz::{
        cell::Intensity,
        escape::{
            CSI,
            csi::{Cursor as CsiCursor, Sgr},
        },
        surface::Surface,
    };

    fn make_surface() -> Surface {
        Surface::new(10, 3)
    }

    #[test]
    fn csi_sgr_applies_intensity() {
        let mut s = make_surface();
        let csi = CSI::Sgr(Sgr::Intensity(Intensity::Bold));
        process_csi(&mut s, &mut std::io::sink(), &csi);
        s.add_change("A");
        let screen = s.screen_cells();
        let cell = &screen[0][0];
        assert_eq!(cell.attrs().intensity(), Intensity::Bold);
    }

    #[test]
    fn csi_cursor_moves_right() {
        let mut s = make_surface();
        s.add_change("X");
        let csi = CSI::Cursor(CsiCursor::Right(3));
        process_csi(&mut s, &mut std::io::sink(), &csi);
        let (x, y) = s.cursor_position();
        assert_eq!(x, 4);
        assert_eq!(y, 0);
    }

    #[test]
    fn csi_edit_erase_character() {
        let mut s = make_surface();
        s.add_change("ABCDE");
        let csi = CSI::Edit(Edit::EraseCharacter(3));
        process_csi(&mut s, &mut std::io::sink(), &csi);
        let content = s.screen_chars_to_string();
        println!("{content:?}");
        assert!(content.starts_with("ABCDE"));
    }
}
