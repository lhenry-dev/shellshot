use std::io::{self};

use termwiz::{
    escape::Action,
    surface::{SEQ_ZERO, SequenceNo, Surface},
};
use tracing::debug;

use crate::terminal_builder::action::{
    control::process_control,
    csi::process_csi,
    operating_system_command::process_operating_system_command,
    print::{process_print, process_print_string},
};

mod control;
mod csi;
mod operating_system_command;
mod print;

pub fn process_action(
    surface: &mut Surface,
    writer: &mut dyn io::Write,
    action: &Action,
) -> SequenceNo {
    debug!("Processing action: {:?}", action);

    match action {
        Action::Print(ch) => process_print(surface, *ch),
        Action::PrintString(str) => process_print_string(surface, str),
        Action::Control(control_code) => process_control(surface, *control_code),
        Action::DeviceControl(_device_control_mode) => SEQ_ZERO,
        Action::OperatingSystemCommand(operating_system_command) => {
            process_operating_system_command(surface, writer, operating_system_command)
        }
        Action::CSI(csi) => process_csi(surface, writer, csi),
        Action::Esc(_esc) => SEQ_ZERO,
        Action::Sixel(_sixel) => SEQ_ZERO,
        Action::XtGetTcap(_items) => SEQ_ZERO,
        Action::KittyImage(_kitty_image) => SEQ_ZERO,
    }
}
