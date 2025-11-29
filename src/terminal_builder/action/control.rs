use termwiz::{
    escape::ControlCode,
    surface::{Change, Position, SequenceNo, Surface, SEQ_ZERO},
};

use crate::terminal_builder::utils::tabulate;

pub fn process_control(surface: &mut Surface, control_code: ControlCode) -> SequenceNo {
    match control_code {
        ControlCode::LineFeed | ControlCode::VerticalTab | ControlCode::FormFeed => {
            surface.add_change("\r\n")
        }
        ControlCode::CarriageReturn => surface.add_change("\r"),
        ControlCode::HorizontalTab => surface.add_change(Change::CursorPosition {
            x: Position::Absolute(tabulate(surface.cursor_position().0, 1)),
            y: Position::Relative(0),
        }),
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
        ControlCode::NEL => surface.add_change("\r\n"),
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
        | ControlCode::StartOfHeading
        | ControlCode::StartOfText
        | ControlCode::EndOfText
        | ControlCode::EndOfTransmission
        | ControlCode::Enquiry
        | ControlCode::Acknowledge
        | ControlCode::Bell
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
        | ControlCode::UnitSeparator
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
