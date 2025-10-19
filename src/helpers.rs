//! Some helper functions.

use iced::{gradient::{ColorStop, Linear}, Background, Color, Gradient};

/// Adds a [`Color`] on top of an other one.
pub fn filter_color(color: Color, filter: Color) -> Color {
    let ac = color.a;
    let af = filter.a;

    let at = af + ac * (1.0 - af);

    let aux = |c, f| (c * ac * (1.0 - af) + f * af) / at;

    Color::from_rgba(
        aux(color.r, filter.r),
        aux(color.g, filter.g),
        aux(color.b, filter.b),
        at,
    )
}

/// Adds a [`Color`] on top of a [`Background`].
pub fn filter_background(background: Background, filter: Color) -> Background {
    match background {
        iced::Background::Color(color) => Background::Color(filter_color(color, filter)),
        iced::Background::Gradient(gradient) => match gradient {
            iced::Gradient::Linear(linear) => {
                let new_stops = linear.stops.map(|x| {
                    x.map(|stop| ColorStop {
                        color: filter_color(stop.color, filter),
                        ..stop
                    })
                });

                Background::Gradient(Gradient::Linear(Linear {
                    stops: new_stops,
                    ..linear
                }))
            }
        },
    }
}
