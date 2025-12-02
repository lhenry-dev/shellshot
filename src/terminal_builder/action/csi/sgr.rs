use termwiz::{
    cell::AttributeChange,
    escape::csi::Sgr,
    surface::{Change, SequenceNo, Surface, SEQ_ZERO},
};

pub fn process_sgr(surface: &mut Surface, sgr: &Sgr) -> SequenceNo {
    match sgr {
        Sgr::Reset => surface.add_change(Change::AllAttributes(Default::default())),
        Sgr::Intensity(intensity) => {
            surface.add_change(Change::Attribute(AttributeChange::Intensity(*intensity)))
        }
        Sgr::Underline(underline) => {
            surface.add_change(Change::Attribute(AttributeChange::Underline(*underline)))
        }
        Sgr::Inverse(inverse) => {
            surface.add_change(Change::Attribute(AttributeChange::Reverse(*inverse)))
        }
        Sgr::Foreground(color) => surface.add_change(Change::Attribute(
            AttributeChange::Foreground((*color).into()),
        )),
        Sgr::Background(color) => surface.add_change(Change::Attribute(
            AttributeChange::Background((*color).into()),
        )),
        Sgr::Italic(italic) => {
            surface.add_change(Change::Attribute(AttributeChange::Italic(*italic)))
        }
        Sgr::StrikeThrough(enabled) => {
            surface.add_change(Change::Attribute(AttributeChange::StrikeThrough(*enabled)))
        }
        Sgr::Invisible(enabled) => {
            surface.add_change(Change::Attribute(AttributeChange::Invisible(*enabled)))
        }
        Sgr::UnderlineColor(_)
        | Sgr::Blink(_)
        | Sgr::Font(_)
        | Sgr::Overline(_)
        | Sgr::VerticalAlign(_) => SEQ_ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termwiz::cell::{Intensity, Underline};
    use termwiz::color::{ColorAttribute, ColorSpec, SrgbaTuple};
    use termwiz::escape::csi::Sgr;
    use termwiz::surface::Surface;

    fn make_surface() -> Surface {
        Surface::new(80, 24)
    }

    fn apply_sgr(surface: &mut Surface, sgr: Sgr) {
        process_sgr(surface, &sgr);
    }

    #[test]
    fn sgr_reset() {
        let mut s = make_surface();
        apply_sgr(&mut s, Sgr::Reset);

        s.add_change("A");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];

        assert_eq!(first_cell.attrs().intensity(), Intensity::Normal);
        assert_eq!(first_cell.attrs().underline(), Underline::None);
        assert_eq!(s.cursor_shape(), None);
    }

    #[test]
    fn sgr_intensity() {
        let mut s = make_surface();
        apply_sgr(&mut s, Sgr::Intensity(Intensity::Bold));

        s.add_change("B");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];

        assert_eq!(first_cell.attrs().intensity(), Intensity::Bold);
    }

    #[test]
    fn sgr_underline() {
        let mut s = make_surface();
        apply_sgr(&mut s, Sgr::Underline(Underline::Single));

        s.add_change("C");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];
        assert_eq!(first_cell.attrs().underline(), Underline::Single);

        apply_sgr(&mut s, Sgr::Underline(Underline::Double));
        s.add_change("D");
        let screen = s.screen_cells();
        let second_cell = &screen[0][1];
        assert_eq!(second_cell.attrs().underline(), Underline::Double);

        apply_sgr(&mut s, Sgr::Underline(Underline::None));
        s.add_change("E");
        let screen = s.screen_cells();
        let third_cell = &screen[0][2];
        assert_eq!(third_cell.attrs().underline(), Underline::None);
    }

    #[test]
    fn sgr_inverse() {
        let mut s = make_surface();
        apply_sgr(&mut s, Sgr::Inverse(true));

        s.add_change("F");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];
        assert!(first_cell.attrs().reverse());

        apply_sgr(&mut s, Sgr::Inverse(false));
        s.add_change("G");
        let screen = s.screen_cells();
        let second_cell = &screen[0][1];
        assert!(!second_cell.attrs().reverse());
    }

    #[test]
    fn sgr_italic() {
        let mut s = make_surface();
        apply_sgr(&mut s, Sgr::Italic(true));

        s.add_change("H");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];
        assert!(first_cell.attrs().italic());
    }

    #[test]
    fn sgr_strikethrough() {
        let mut s = make_surface();
        apply_sgr(&mut s, Sgr::StrikeThrough(true));

        s.add_change("I");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];
        assert!(first_cell.attrs().strikethrough());
    }

    #[test]
    fn sgr_invisible() {
        let mut s = make_surface();
        apply_sgr(&mut s, Sgr::Invisible(true));

        s.add_change("J");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];
        assert!(first_cell.attrs().invisible());
    }

    #[test]
    fn sgr_foreground_background() {
        let mut s = make_surface();
        apply_sgr(&mut s, Sgr::Foreground(ColorSpec::PaletteIndex(120)));

        s.add_change("K");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];
        assert_eq!(
            first_cell.attrs().foreground(),
            ColorAttribute::PaletteIndex(120)
        );

        apply_sgr(&mut s, Sgr::Background(ColorSpec::Default));
        s.add_change("L");
        let screen = s.screen_cells();
        let second_cell = &screen[0][1];
        assert_eq!(second_cell.attrs().background(), ColorAttribute::Default);
    }

    #[test]
    fn sgr_foreground_truecolor() {
        let mut s = make_surface();
        let color = SrgbaTuple(1.0, 0.0, 0.0, 1.0);
        apply_sgr(&mut s, Sgr::Foreground(ColorSpec::TrueColor(color)));

        s.add_change("M");
        let screen = s.screen_cells();
        let first_cell = &screen[0][0];
        assert_eq!(
            first_cell.attrs().foreground(),
            ColorAttribute::TrueColorWithDefaultFallback(color)
        );
    }
}
