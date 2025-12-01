use std::io::{self};

use num_traits::FromPrimitive;
use termwiz::{
    cell::AttributeChange,
    color::ColorAttribute,
    escape::{
        osc::{ColorOrQuery, DynamicColorNumber},
        OperatingSystemCommand,
    },
    surface::{Change, SequenceNo, Surface, SEQ_ZERO},
};

pub fn process_operating_system_command(
    surface: &mut Surface,
    writer: &mut dyn io::Write,
    operating_system_command: &OperatingSystemCommand,
) -> SequenceNo {
    match operating_system_command {
        OperatingSystemCommand::ChangeDynamicColors(dynamic_color_number, items) => {
            process_change_dynamic_colors(surface, writer, dynamic_color_number, items.to_vec())
        }
        OperatingSystemCommand::ResetDynamicColor(dynamic_color_number) => {
            process_reset_dynamic_color(surface, dynamic_color_number)
        }
        OperatingSystemCommand::ResetColors(items) => {
            for byte in items {
                if let Some(color) = &FromPrimitive::from_u8(*byte) {
                    process_reset_dynamic_color(surface, color);
                }
            }

            SEQ_ZERO
        }
        OperatingSystemCommand::SetIconNameAndWindowTitle(_)
        | OperatingSystemCommand::SetWindowTitle(_)
        | OperatingSystemCommand::SetWindowTitleSun(_)
        | OperatingSystemCommand::SetIconName(_)
        | OperatingSystemCommand::SetIconNameSun(_)
        | OperatingSystemCommand::SetHyperlink(_)
        | OperatingSystemCommand::ClearSelection(_)
        | OperatingSystemCommand::QuerySelection(_)
        | OperatingSystemCommand::SetSelection(_, _)
        | OperatingSystemCommand::SystemNotification(_)
        | OperatingSystemCommand::ITermProprietary(_)
        | OperatingSystemCommand::FinalTermSemanticPrompt(_)
        | OperatingSystemCommand::ChangeColorNumber(_)
        | OperatingSystemCommand::CurrentWorkingDirectory(_)
        | OperatingSystemCommand::RxvtExtension(_)
        | OperatingSystemCommand::ConEmuProgress(_)
        | OperatingSystemCommand::Unspecified(_) => SEQ_ZERO,
    }
}

fn process_change_dynamic_colors(
    surface: &mut Surface,
    writer: &mut dyn io::Write,
    first_color: &DynamicColorNumber,
    colors: Vec<ColorOrQuery>,
) -> SequenceNo {
    colors
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let idx = *first_color as u8 + i as u8;
            FromPrimitive::from_u8(idx).map(|dc| (dc, c))
        })
        .for_each(|(target, color)| match target {
            DynamicColorNumber::TextForegroundColor => {
                if let Some(attr) = color_or_query(writer, target, color) {
                    surface.add_change(Change::Attribute(AttributeChange::Foreground(attr)));
                }
            }
            DynamicColorNumber::TextBackgroundColor => {
                if let Some(attr) = color_or_query(writer, target, color) {
                    surface.add_change(Change::Attribute(AttributeChange::Background(attr)));
                }
            }
            DynamicColorNumber::TextCursorColor => unimplemented!(),
            _ => unimplemented!(),
        });

    SEQ_ZERO
}

fn process_reset_dynamic_color(
    surface: &mut Surface,
    dynamic_color_number: &DynamicColorNumber,
) -> SequenceNo {
    let idx: u8 = *dynamic_color_number as u8;

    if let Some(which_color) = FromPrimitive::from_u8(idx) {
        match which_color {
            DynamicColorNumber::TextForegroundColor => {
                surface.add_change(Change::Attribute(AttributeChange::Foreground(
                    ColorAttribute::Default,
                )));
            }
            DynamicColorNumber::TextBackgroundColor => {
                surface.add_change(Change::Attribute(AttributeChange::Background(
                    ColorAttribute::Default,
                )));
            }
            DynamicColorNumber::TextCursorColor => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    SEQ_ZERO
}

fn color_or_query(
    writer: &mut dyn io::Write,
    target: DynamicColorNumber,
    color: ColorOrQuery,
) -> Option<ColorAttribute> {
    match color {
        ColorOrQuery::Color(c) => Some(ColorAttribute::TrueColorWithDefaultFallback(c)),
        ColorOrQuery::Query => {
            let response = OperatingSystemCommand::ChangeDynamicColors(target, vec![color]);
            write!(writer, "{response}").ok();
            writer.flush().ok();
            None
        }
    }
}
