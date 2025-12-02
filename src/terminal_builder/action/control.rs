use termwiz::{
    escape::ControlCode,
    surface::{Change, Position, SequenceNo, Surface, SEQ_ZERO},
};

pub fn process_control(surface: &mut Surface, control_code: ControlCode) -> SequenceNo {
    match control_code {
        ControlCode::LineFeed
        | ControlCode::VerticalTab
        | ControlCode::FormFeed
        | ControlCode::CarriageReturn
        | ControlCode::HorizontalTab
        | ControlCode::ShiftOut
        | ControlCode::ShiftIn
        | ControlCode::DataLinkEscape
        | ControlCode::DeviceControlOne
        | ControlCode::DeviceControlTwo
        | ControlCode::DeviceControlThree
        | ControlCode::DeviceControlFour
        | ControlCode::NegativeAcknowledge
        | ControlCode::SynchronousIdle
        | ControlCode::EndOfTransmissionBlock
        | ControlCode::Cancel
        | ControlCode::EndOfMedium
        | ControlCode::Substitute
        | ControlCode::Escape
        | ControlCode::FileSeparator
        | ControlCode::GroupSeparator
        | ControlCode::RecordSeparator
        | ControlCode::UnitSeparator => surface.add_change(control_code as u8 as char),

        ControlCode::Backspace => {
            surface.add_change(Change::CursorPosition {
                x: Position::Relative(-1),
                y: Position::Relative(0),
            });
            surface.add_change(" ");
            surface.add_change(Change::CursorPosition {
                x: Position::Relative(-1),
                y: Position::Relative(0),
            })
        }
        ControlCode::RI => {
            let (x, y) = surface.cursor_position();
            if y == 0 {
                surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(x),
                    y: Position::Absolute(0),
                })
            } else {
                surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(x),
                    y: Position::Relative(-1),
                })
            }
        }
        ControlCode::Null
        | ControlCode::NEL
        | ControlCode::StartOfHeading
        | ControlCode::StartOfText
        | ControlCode::EndOfText
        | ControlCode::EndOfTransmission
        | ControlCode::Enquiry
        | ControlCode::Acknowledge
        | ControlCode::Bell
        | ControlCode::BPH
        | ControlCode::NBH
        | ControlCode::IND
        | ControlCode::SSA
        | ControlCode::ESA
        | ControlCode::HTS
        | ControlCode::HTJ
        | ControlCode::VTS
        | ControlCode::PLD
        | ControlCode::PLU
        | ControlCode::SS2
        | ControlCode::SS3
        | ControlCode::DCS
        | ControlCode::PU1
        | ControlCode::PU2
        | ControlCode::STS
        | ControlCode::CCH
        | ControlCode::MW
        | ControlCode::SPA
        | ControlCode::EPA
        | ControlCode::SOS
        | ControlCode::SCI
        | ControlCode::CSI
        | ControlCode::ST
        | ControlCode::OSC
        | ControlCode::PM
        | ControlCode::APC => SEQ_ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termwiz::escape::ControlCode;
    use termwiz::surface::{Change, Position, Surface};

    fn make_surface() -> Surface {
        Surface::new(10, 3)
    }

    fn apply_control(surface: &mut Surface, code: ControlCode) -> SequenceNo {
        process_control(surface, code)
    }

    #[test]
    fn test_linefeed() {
        let mut s = make_surface();
        let (_, y_before) = s.cursor_position();
        apply_control(&mut s, ControlCode::LineFeed);
        let (_, y_after) = s.cursor_position();
        assert_eq!(y_after, y_before + 1);
    }

    #[test]
    fn test_carriage_return() {
        let mut s = make_surface();
        s.add_change("ABCDE");
        apply_control(&mut s, ControlCode::CarriageReturn);
        let (x, _) = s.cursor_position();
        assert_eq!(x, 0);
    }

    #[test]
    fn test_backspace() {
        let mut s = make_surface();
        s.add_change("X");
        let before = s.cursor_position();
        apply_control(&mut s, ControlCode::Backspace);
        let after = s.cursor_position();
        assert_eq!(after.0, before.0.saturating_sub(1));
    }

    #[test]
    fn test_ri_at_top() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(2),
            y: Position::Absolute(0),
        });
        apply_control(&mut s, ControlCode::RI);
        let (_, y) = s.cursor_position();
        assert_eq!(y, 0);
    }

    #[test]
    fn test_ri_not_at_top() {
        let mut s = make_surface();
        s.add_change(Change::CursorPosition {
            x: Position::Absolute(1),
            y: Position::Absolute(2),
        });
        let before_y = s.cursor_position().1;
        apply_control(&mut s, ControlCode::RI);
        let (_, after_y) = s.cursor_position();
        assert_eq!(after_y, before_y - 1);
    }
}
