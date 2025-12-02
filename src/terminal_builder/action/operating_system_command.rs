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
            DynamicColorNumber::TextCursorColor
            | DynamicColorNumber::MouseForegroundColor
            | DynamicColorNumber::MouseBackgroundColor
            | DynamicColorNumber::TektronixForegroundColor
            | DynamicColorNumber::TektronixBackgroundColor
            | DynamicColorNumber::HighlightBackgroundColor
            | DynamicColorNumber::TektronixCursorColor
            | DynamicColorNumber::HighlightForegroundColor => (),
        });

    SEQ_ZERO
}

fn process_reset_dynamic_color(
    surface: &mut Surface,
    dynamic_color_number: &DynamicColorNumber,
) -> SequenceNo {
    let idx: u8 = *dynamic_color_number as u8;

    if let Some(which_color) = FromPrimitive::from_u8(idx) {
        return match which_color {
            DynamicColorNumber::TextForegroundColor => surface.add_change(Change::Attribute(
                AttributeChange::Foreground(ColorAttribute::Default),
            )),
            DynamicColorNumber::TextBackgroundColor => surface.add_change(Change::Attribute(
                AttributeChange::Background(ColorAttribute::Default),
            )),
            DynamicColorNumber::TextCursorColor
            | DynamicColorNumber::MouseForegroundColor
            | DynamicColorNumber::MouseBackgroundColor
            | DynamicColorNumber::TektronixForegroundColor
            | DynamicColorNumber::TektronixBackgroundColor
            | DynamicColorNumber::HighlightBackgroundColor
            | DynamicColorNumber::TektronixCursorColor
            | DynamicColorNumber::HighlightForegroundColor => SEQ_ZERO,
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use termwiz::escape::osc::{ColorOrQuery, DynamicColorNumber};
    use termwiz::escape::OperatingSystemCommand;
    use termwiz::{
        color::{ColorAttribute, SrgbaTuple},
        surface::Surface,
    };

    fn make_surface() -> Surface {
        Surface::new(10, 1)
    }

    fn apply_osc(surface: &mut Surface, osc: OperatingSystemCommand) {
        let mut writer = std::io::sink();
        process_operating_system_command(surface, &mut writer, &osc);
    }

    #[test]
    fn test_set_foreground_color() {
        let mut s = make_surface();
        let color = SrgbaTuple(1.0, 0.0, 0.0, 1.0);

        apply_osc(
            &mut s,
            OperatingSystemCommand::ChangeDynamicColors(
                DynamicColorNumber::TextForegroundColor,
                vec![ColorOrQuery::Color(color)],
            ),
        );

        s.add_change("A");
        let screen = s.screen_cells();
        let cell = &screen[0][0];
        assert_eq!(
            cell.attrs().foreground(),
            ColorAttribute::TrueColorWithDefaultFallback(color)
        );
    }

    #[test]
    fn test_set_background_color() {
        let mut s = make_surface();
        let color = SrgbaTuple(0.0, 1.0, 0.0, 1.0);

        apply_osc(
            &mut s,
            OperatingSystemCommand::ChangeDynamicColors(
                DynamicColorNumber::TextBackgroundColor,
                vec![ColorOrQuery::Color(color)],
            ),
        );

        s.add_change("B");
        let screen = s.screen_cells();
        let cell = &screen[0][0];
        assert_eq!(
            cell.attrs().background(),
            ColorAttribute::TrueColorWithDefaultFallback(color)
        );
    }

    #[test]
    fn test_reset_foreground_color() {
        let mut s = make_surface();
        let color = SrgbaTuple(1.0, 0.0, 0.0, 1.0);

        apply_osc(
            &mut s,
            OperatingSystemCommand::ChangeDynamicColors(
                DynamicColorNumber::TextForegroundColor,
                vec![ColorOrQuery::Color(color)],
            ),
        );

        apply_osc(
            &mut s,
            OperatingSystemCommand::ResetDynamicColor(DynamicColorNumber::TextForegroundColor),
        );

        s.add_change("C");
        let screen = s.screen_cells();
        let cell = &screen[0][0];
        assert_eq!(cell.attrs().foreground(), ColorAttribute::Default);
    }

    #[test]
    fn test_reset_background_color() {
        let mut s = make_surface();
        let color = SrgbaTuple(0.0, 1.0, 0.0, 1.0);

        apply_osc(
            &mut s,
            OperatingSystemCommand::ChangeDynamicColors(
                DynamicColorNumber::TextBackgroundColor,
                vec![ColorOrQuery::Color(color)],
            ),
        );

        apply_osc(
            &mut s,
            OperatingSystemCommand::ResetDynamicColor(DynamicColorNumber::TextBackgroundColor),
        );

        s.add_change("D");
        let screen = s.screen_cells();
        let cell = &screen[0][0];
        assert_eq!(cell.attrs().background(), ColorAttribute::Default);
    }

    #[test]
    fn test_reset_colors_multiple() {
        let mut s = make_surface();
        let color = SrgbaTuple(0.0, 1.0, 0.0, 1.0);

        apply_osc(
            &mut s,
            OperatingSystemCommand::ChangeDynamicColors(
                DynamicColorNumber::TextBackgroundColor,
                vec![ColorOrQuery::Color(color)],
            ),
        );

        apply_osc(
            &mut s,
            OperatingSystemCommand::ResetColors(vec![
                DynamicColorNumber::TextForegroundColor as u8,
                DynamicColorNumber::TextBackgroundColor as u8,
            ]),
        );

        s.add_change("E");
        let screen = s.screen_cells();
        let cell = &screen[0][0];
        assert_eq!(cell.attrs().foreground(), ColorAttribute::Default);
        assert_eq!(cell.attrs().background(), ColorAttribute::Default);
    }

    #[test]
    fn test_color_or_query_query_does_not_modify_cell() {
        let mut s = make_surface();

        apply_osc(
            &mut s,
            OperatingSystemCommand::ChangeDynamicColors(
                DynamicColorNumber::TextForegroundColor,
                vec![ColorOrQuery::Query],
            ),
        );

        s.add_change("F");
        let screen = s.screen_cells();
        let cell = &screen[0][0];

        assert_eq!(cell.attrs().foreground(), ColorAttribute::Default);
        assert_eq!(cell.attrs().background(), ColorAttribute::Default);
    }
}
