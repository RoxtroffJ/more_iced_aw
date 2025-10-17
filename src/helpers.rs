//! Some helper functions.

use iced::Color;

/// Adds a [`Color`] on top of an other one.
pub fn filter_color(color: Color, filter: Color) -> Color {
    let ac = color.a;
    let af = filter.a;

    let at = af + ac * (1.0 - af);

    let aux = |c, f| (c * ac * (1.0 - af) + f * af) / at;

    Color::from_linear_rgba(
        aux(color.r, filter.r),
        aux(color.g, filter.g),
        aux(color.b, filter.b),
        at,
    )
}
