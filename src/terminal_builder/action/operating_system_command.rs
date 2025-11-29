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
    operating_system_command: OperatingSystemCommand,
) -> SequenceNo {
    match operating_system_command {
        OperatingSystemCommand::ChangeDynamicColors(dynamic_color_number, items) => {
            process_change_dynamic_colors(surface, dynamic_color_number, items)
        }
        OperatingSystemCommand::ResetDynamicColor(_dynamic_color_number) => todo!(),
        OperatingSystemCommand::ResetColors(_items) => todo!(),
        _ => SEQ_ZERO,
    }
}

fn process_change_dynamic_colors(
    surface: &mut Surface,
    first_color: DynamicColorNumber,
    colors: Vec<ColorOrQuery>,
) -> SequenceNo {
    let mut idx: u8 = first_color as u8;

    for color in colors {
        if let Some(which_color) = FromPrimitive::from_u8(idx) {
            match which_color {
                DynamicColorNumber::TextForegroundColor => {
                    surface.add_change(Change::Attribute(AttributeChange::Foreground(
                        color_or_query_to_color_attribute(color),
                    )));
                }
                DynamicColorNumber::TextBackgroundColor => {
                    surface.add_change(Change::Attribute(AttributeChange::Background(
                        color_or_query_to_color_attribute(color),
                    )));
                }
                DynamicColorNumber::TextCursorColor => unimplemented!(),
                _ => unimplemented!(),
            }
        }
        idx += 1;
    }

    SEQ_ZERO
}

fn color_or_query_to_color_attribute(color: ColorOrQuery) -> ColorAttribute {
    match color {
        ColorOrQuery::Color(c) => ColorAttribute::TrueColorWithDefaultFallback(c),
        ColorOrQuery::Query => ColorAttribute::Default,
    }
}
