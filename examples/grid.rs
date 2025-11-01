use std::{
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    ops::Deref,
};

use iced::{
    Element,
    Length::{self, *},
    alignment::{
        Horizontal::{self, Left},
        Vertical::{self, Top},
    },
    color,
    widget::{
        ComboBox, Row, center, checkbox, column, combo_box, container, row, text_input,
        vertical_rule,
    },
};
use more_iced_aw::{
    grid,
    parsed_input::{self, ParsedInput},
};

// An interactive grid to showcase how grids works.

fn main() -> iced::Result {
    iced::run("Grid", App::update, App::view)
}

/// A cell the bounds of which are clearly visible.
///
/// Provides buttons to change it's dimensions.
#[derive(Debug, Clone)]
struct Cell {
    width: DispLength,
    height: DispLength,

    width_float_parsed: parsed_input::Content<f32, ParseFloatError>,
    height_float_parsed: parsed_input::Content<f32, ParseFloatError>,

    width_int_parsed: parsed_input::Content<u16, ParseIntError>,
    height_int_parsed: parsed_input::Content<u16, ParseIntError>,

    length_state: combo_box::State<DispLength>,
}

#[derive(Debug, Clone)]
enum CellAction {
    EditWidth(DispLength),
    EditHeight(DispLength),

    WidthFloatParsed(parsed_input::Parsed<f32, ParseFloatError>),
    HeightFloatParsed(parsed_input::Parsed<f32, ParseFloatError>),

    WidthIntParsed(parsed_input::Parsed<u16, ParseIntError>),
    HeightIntParsed(parsed_input::Parsed<u16, ParseIntError>),
}

impl Cell {
    fn width_line<'a, Renderer: iced::advanced::text::Renderer + 'a>(&'a self) -> Row<'a, CellAction, iced::Theme, Renderer> {
        let width_combo_box = ComboBox::new(
            &self.length_state,
            "",
            Some(&self.width),
            CellAction::EditWidth,
        )
        .width(100);
        row!["Width:", width_combo_box]
            .push_maybe(match &*self.width {
                FillPortion(_) => Some(Element::from(
                    ParsedInput::new("Fill Portion", &self.width_int_parsed)
                        .style(parsed_input::danger_on_err(text_input::default))
                        .on_input(CellAction::WidthIntParsed)
                        .width(50),
                )),
                Fixed(_) => Some(
                    ParsedInput::new("Width", &self.width_float_parsed)
                        .style(parsed_input::danger_on_err(text_input::default))
                        .on_input(CellAction::WidthFloatParsed)
                        .width(50)
                        .into(),
                ),
                _ => None,
            })
            .spacing(10)
            .align_y(Vertical::Center)
    }

    fn height_line<'a, Renderer: iced::advanced::text::Renderer + 'a>(&'a self) -> Row<'a, CellAction, iced::Theme, Renderer> {
        let height_combo_box = ComboBox::new(
            &self.length_state,
            "",
            Some(&self.height),
            CellAction::EditHeight,
        )
        .width(100);

        row!["Height:", height_combo_box]
            .push_maybe(match &*self.height {
                FillPortion(_) => Some(Element::from(
                    ParsedInput::new("Fill Portion", &self.height_int_parsed)
                        .style(parsed_input::danger_on_err(text_input::default))
                        .on_input(CellAction::HeightIntParsed)
                        .width(50),
                )),
                Fixed(_) => Some(
                    ParsedInput::new("Height", &self.height_float_parsed)
                        .style(parsed_input::danger_on_err(text_input::default))
                        .on_input(CellAction::HeightFloatParsed)
                        .width(50)
                        .into(),
                ),
                _ => None,
            })
            .spacing(10)
            .align_y(Vertical::Center)
    }

    fn to_element<'a>(
        &'a self,
        on_action: impl Fn(CellAction) -> Message + 'a,
    ) -> Element<'a, Message> {
        let elt: Element<'_, _> = center(column![self.width_line(), self.height_line()])
            .width(match *self.width {
                FillPortion(_) => FillPortion(*self.width_int_parsed),
                Fixed(_) => Fixed(*self.width_float_parsed + 200.),
                _ => self.width.into(),
            })
            .height(match *self.height {
                FillPortion(_) => FillPortion(*self.height_int_parsed),
                Fixed(_) => Fixed(*self.height_float_parsed),
                _ => self.height.into(),
            })
            .style(container::bordered_box)
            .into();

        elt.map(on_action)
    }

    fn perform(&mut self, action: CellAction) {
        match action {
            CellAction::EditWidth(disp_length) => self.width = disp_length,
            CellAction::EditHeight(disp_length) => self.height = disp_length,
            CellAction::WidthFloatParsed(parsed) => self.width_float_parsed.update(parsed),
            CellAction::HeightFloatParsed(parsed) => self.height_float_parsed.update(parsed),
            CellAction::WidthIntParsed(parsed) => self.width_int_parsed.update(parsed),
            CellAction::HeightIntParsed(parsed) => self.height_int_parsed.update(parsed),
        }
    }
}

struct App {
    grid: Vec<Vec<Cell>>,

    cell: Cell,

    padding: parsed_input::Content<f32, ParseFloatError>,
    align_x: DispHorizontal,
    align_y: DispVertical,

    column_spacing: parsed_input::Content<f32, ParseFloatError>,
    row_spacing: parsed_input::Content<f32, ParseFloatError>,
    axis: grid::Axis,

    horiz_state: combo_box::State<DispHorizontal>,
    verti_state: combo_box::State<DispVertical>,
    axis_state: combo_box::State<grid::Axis>,

    explain: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Forward(usize, usize, CellAction),
    Cell(CellAction),

    Padding(parsed_input::Parsed<f32, ParseFloatError>),
    AlignX(DispHorizontal),
    AlignY(DispVertical),

    ColumnSpacing(parsed_input::Parsed<f32, ParseFloatError>),
    RowSpacing(parsed_input::Parsed<f32, ParseFloatError>),

    Axis(grid::Axis),

    Explain(bool),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Forward(i, j, cell_action) => self.grid[i][j].perform(cell_action),
            Message::Cell(cell_action) => self.cell.perform(cell_action),
            Message::Padding(parsed) => self.padding.update(parsed),
            Message::AlignX(horizontal) => self.align_x = horizontal,
            Message::AlignY(vertical) => self.align_y = vertical,
            Message::ColumnSpacing(parsed) => self.column_spacing.update(parsed),
            Message::RowSpacing(parsed) => self.row_spacing.update(parsed),
            Message::Axis(axis) => self.axis = axis,
            Message::Explain(bool) => self.explain = bool,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut grid: Element<'_, _> = grid::Grid::new()
            .extend(self.grid.iter().enumerate().map(|(i, vec)| {
                vec.iter().enumerate().map(move |(j, cell)| {
                    cell.to_element(move |action| Message::Forward(i, j, action))
                })
            }))
            .width(match *self.cell.width {
                FillPortion(_) => FillPortion(*self.cell.width_int_parsed),
                Fixed(_) => Fixed(*self.cell.width_float_parsed),
                _ => self.cell.width.into(),
            })
            .height(match *self.cell.height {
                FillPortion(_) => FillPortion(*self.cell.height_int_parsed),
                Fixed(_) => Fixed(*self.cell.height_float_parsed),
                _ => self.cell.height.into(),
            })
            .padding(*self.padding)
            .align_x(self.align_x)
            .align_y(self.align_y)
            .column_spacing(*self.column_spacing)
            .row_spacing(*self.row_spacing)
            .main_axis(self.axis)
            .into();

        if self.explain {
            grid = grid.explain(color!(0xff0000))
        }

        let side_panel = column![
            Element::from(self.cell.width_line()).map(Message::Cell),
            Element::from(self.cell.height_line()).map(Message::Cell),
            row![
                "Padding",
                parsed_input::ParsedInput::new("Padding", &self.padding)
                    .on_input(Message::Padding)
                    .style(parsed_input::danger_on_err(text_input::default)),
            ]
            .spacing(10),
            row![
                "Align x",
                combo_box::ComboBox::new(
                    &self.horiz_state,
                    "",
                    Some(&self.align_x),
                    Message::AlignX
                ),
            ]
            .spacing(10),
            row![
                "Align y",
                combo_box::ComboBox::new(
                    &self.verti_state,
                    "",
                    Some(&self.align_y),
                    Message::AlignY
                ),
            ]
            .spacing(10),
            row![
                "Column spacing",
                parsed_input::ParsedInput::new("Column spacing", &self.column_spacing)
                    .on_input(Message::ColumnSpacing)
                    .style(parsed_input::danger_on_err(text_input::default)),
            ]
            .spacing(10),
            row![
                "Row spacing",
                parsed_input::ParsedInput::new("Row spacing", &self.row_spacing)
                    .on_input(Message::RowSpacing)
                    .style(parsed_input::danger_on_err(text_input::default)),
            ]
            .spacing(10),
            row![
                "Main axis",
                combo_box::ComboBox::new(&self.axis_state, "", Some(&self.axis), Message::Axis),
            ]
            .spacing(10),
            row![
                "Explain",
                checkbox("", self.explain).on_toggle(Message::Explain)
            ]
            .spacing(10),
        ]
        .spacing(10)
        .width(300)
        .padding(10);

        row![side_panel, vertical_rule(10), grid].into()
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            width: Shrink.into(),
            height: Shrink.into(),

            width_float_parsed: parsed_input::Content::new(0.),
            height_float_parsed: parsed_input::Content::new(0.),
            width_int_parsed: parsed_input::Content::new(1),
            height_int_parsed: parsed_input::Content::new(1),

            length_state: combo_box::State::new(LENGTH_OPTIONS.to_vec()),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            grid: vec![
                vec![Cell::default(); 3],
                vec![Cell::default(); 2],
                vec![Cell::default(); 4],
            ],
            cell: Default::default(),
            padding: Default::default(),
            align_x: Left.into(),
            align_y: Top.into(),
            column_spacing: Default::default(),
            row_spacing: Default::default(),
            axis: grid::Axis::Horizontal,
            horiz_state: combo_box::State::new(vec![
                Horizontal::Left.into(),
                Horizontal::Center.into(),
                Horizontal::Right.into(),
            ]),
            verti_state: combo_box::State::new(
                vec![
                    Vertical::Top.into(),
                    Vertical::Center.into(),
                    Vertical::Bottom.into(),
                ]
                .into(),
            ),
            axis_state: combo_box::State::new(vec![grid::Axis::Horizontal, grid::Axis::Vertical]),

            explain: true,
        }
    }
}

/// A [`Length`] that can be displayed.
#[derive(Debug, Clone, Copy, PartialEq)]
struct DispLength {
    length: Length,
}

impl Display for DispLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.length {
                Length::Fill => "Fill",
                Length::FillPortion(_) => "Fill Portion",
                Shrink => "Shrink",
                Length::Fixed(_) => "Fixed",
            }
        )
    }
}

impl From<Length> for DispLength {
    fn from(value: Length) -> Self {
        Self { length: value }
    }
}

impl From<DispLength> for Length {
    fn from(value: DispLength) -> Self {
        value.length
    }
}

impl Deref for DispLength {
    type Target = Length;

    fn deref(&self) -> &Self::Target {
        &self.length
    }
}

const LENGTH_OPTIONS: [DispLength; 4] = [
    DispLength { length: Shrink },
    DispLength {
        length: Fixed(1f32),
    },
    DispLength {
        length: FillPortion(1),
    },
    DispLength { length: Fill },
];

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
struct DispHorizontal {
    v: Horizontal,
}

impl From<Horizontal> for DispHorizontal {
    fn from(value: Horizontal) -> Self {
        Self { v: value }
    }
}

impl From<DispHorizontal> for Horizontal {
    fn from(value: DispHorizontal) -> Self {
        value.v
    }
}

impl Deref for DispHorizontal {
    type Target = Horizontal;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

impl Display for DispHorizontal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.deref() {
                Left => "Left",
                Horizontal::Center => "Center",
                Horizontal::Right => "Right",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
struct DispVertical {
    v: Vertical,
}

impl From<Vertical> for DispVertical {
    fn from(value: Vertical) -> Self {
        Self { v: value }
    }
}

impl From<DispVertical> for Vertical {
    fn from(value: DispVertical) -> Self {
        value.v
    }
}

impl Deref for DispVertical {
    type Target = Vertical;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

impl Display for DispVertical {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.deref() {
                Top => "Top",
                Vertical::Center => "Center",
                Vertical::Bottom => "Bottom",
            }
        )
    }
}
