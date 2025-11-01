//! Grids for iced 0.13.
//!
//! The implementation is different from the one of iced_aw.
//! Arguably, the column sizing technique is different.
//! The shrink should be better more consistent with [`row`](iced::widget::Row) and [`column`](iced::widget::Column),
//! but this grid implementation is also probably slower.
//!
//! See the `grid` example for an example.

use std::{collections::HashSet, fmt::Display};

use iced::{
    Length::{self, Shrink},
    Padding, Pixels, Point, Size,
    advanced::{
        self, Widget,
        graphics::core::Element,
        layout::{self, Limits, Node},
        widget::Tree,
    },
    alignment::{Horizontal, Vertical},
    event,
};

/// The [Grid] widget.
pub struct Grid<'a, Message, Theme, Renderer> {
    rows: Vec<Vec<Element<'a, Message, Theme, Renderer>>>,
    width: Length,
    height: Length,
    padding: Padding,

    horizontal_align: Horizontal,
    vertical_align: Vertical,

    column_spacing: f32,
    row_spacing: f32,
    axis: Axis,
}

impl<'a, Message, Theme, Renderer> Grid<'a, Message, Theme, Renderer> {
    /// Creates a new empty grid.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            width: Shrink,
            height: Shrink,
            padding: Padding::ZERO,
            horizontal_align: Horizontal::Left,
            vertical_align: Vertical::Center,
            column_spacing: 0.,
            row_spacing: 0.,
            axis: Axis::Horizontal,
        }
    }

    /// Sets the spacing between the columns.
    pub fn column_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.column_spacing = spacing.into().0;
        self
    }

    /// Sets the spacing between the rows.
    pub fn row_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.row_spacing = spacing.into().0;
        self
    }

    /// Sets the padding of the grid.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the grid.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the grid.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the horizontal alignment of the columns.
    pub fn align_x(mut self, horizontal: impl Into<Horizontal>) -> Self {
        self.horizontal_align = horizontal.into();
        self
    }

    /// Sets the vertical alignment of the rows.
    pub fn align_y(mut self, vertical: impl Into<Vertical>) -> Self {
        self.vertical_align = vertical.into();
        self
    }

    /// Sets the main axis of the grid.
    ///
    /// This main axis dictates how the size of the cells are computed.
    /// * [`Axis::Horizontal`] => cells are layed in rows, and then rows are placed on top of each other.
    /// * [`Axis::Vertical`] => cells are layed in columns, and then placed next to each other.
    pub fn main_axis(mut self, axis: impl Into<Axis>) -> Self {
        self.axis = axis.into();
        self
    }

    /// Adds a row to the grid.
    pub fn push_row<E>(mut self, row: impl IntoIterator<Item = E>) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
        Renderer: advanced::Renderer,
    {
        self.push_row_mut(row);
        self
    }

    /// Same as [`push_row`](Self::push_row) but takes a reference to `self`.
    pub fn push_row_mut<E>(&mut self, row: impl IntoIterator<Item = E>)
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
        Renderer: advanced::Renderer,
    {
        let row = row.into_iter().map(Into::into).collect::<Vec<_>>();

        for e in row.iter() {
            let size = e.as_widget().size_hint();

            self.width.enclose(size.width);
            self.height.enclose(size.height);
        }

        self.rows.push(row);
    }

    /// Adds multiple rows to the grid.
    pub fn extend<E, I>(mut self, rows: impl IntoIterator<Item = I>) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
        I: IntoIterator<Item = E>,
        Renderer: advanced::Renderer,
    {
        self.extend_mut(rows);
        self
    }

    /// Same as [`extend`](Self::extend) but takes a reference to `self`.
    pub fn extend_mut<E, I>(&mut self, rows: impl IntoIterator<Item = I>)
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
        I: IntoIterator<Item = E>,
        Renderer: advanced::Renderer,
    {
        rows.into_iter().for_each(|row| self.push_row_mut(row));
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Grid<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        let children: Vec<_> = self.get_elements().collect();
        tree.diff_children(&children);
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        self.get_elements().map(Tree::new).collect()
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &advanced::layout::Limits,
    ) -> advanced::layout::Node {
        // Nomenclature (given for axis == Horizontal):
        // width / height -> main / cross
        // row / column -> prim / sec

        let axis = self.axis;

        let (max_main, max_cross) = {
            let limits = limits
                .height(self.height)
                .width(self.width)
                .shrink(self.padding);

            axis.size_pack(limits.max())
        };

        let (main_length, cross_length) = axis.pack(self.width, self.height);

        let nb_columns = self.rows.iter().fold(0, |len, vec| len.max(vec.len()));
        let nb_rows = self.rows.len();

        let (nb_prim, nb_sec) = axis.pack(nb_rows, nb_columns);
        let (main_spacing, cross_spacing) = axis.pack(self.column_spacing, self.row_spacing);

        let main_total_spacing = main_spacing * nb_sec.saturating_sub(1) as f32;
        let cross_total_spacing = cross_spacing * nb_prim.saturating_sub(1) as f32;

        let main_max = max_main - main_total_spacing;
        let cross_max = max_cross - cross_total_spacing;

        let mut main = main_max;

        let mut sec_main_factor = vec![0; nb_sec];
        let mut prim_cross_factor = vec![0; nb_prim];

        let mut sec_main = vec![0f32; nb_sec];

        // Map trees to elements.
        let mut elts_trees: Vec<Vec<_>> = {
            let mut iter = tree.children.iter_mut();

            self.rows
                .iter()
                .map(|vec| vec.iter().zip(&mut iter).collect())
                .collect()
        };

        // ==== Build prims with as much cross as they want. (It will be restricted later) ====

        // Compute those with non fill main
        for j in 0..nb_sec {
            for i in 0..nb_prim {
                // Get element and tree
                let (a, b) = axis.pack(i, j);
                let (elt, tree) = {
                    match elts_trees.get_mut(a).and_then(|vec| vec.get_mut(b)) {
                        Some(v) => v,
                        None => continue,
                    }
                };

                // Check size and add fills
                let (main_len, cross_len) = {
                    let size = elt.as_widget().size();
                    axis.size_pack(size)
                };

                let main_fill_factor = main_len.fill_factor();
                let cross_fill_factor = cross_len.fill_factor();

                prim_cross_factor[i] = prim_cross_factor[i].max(cross_fill_factor);
                sec_main_factor[j] = sec_main_factor[j].max(main_fill_factor);

                // If fixed main, compute it and update
                if main_fill_factor == 0 {
                    let (max_width, max_height) = axis.pack(main, cross_max);

                    let child_limits = Limits::new(Size::ZERO, Size::new(max_width, max_height));
                    let layout = elt.as_widget().layout(tree, renderer, &child_limits);

                    let main = axis.main(layout.size());

                    sec_main[j] = sec_main[j].max(main);
                }
            }

            main -= sec_main[j];
        }

        // Get the final main of the secs.
        if main_length != Shrink {
            let mut not_clamped: HashSet<_> = (0..nb_sec).collect();
            main = max_main - main_total_spacing;

            let mut fill_sum = sec_main_factor.iter().sum::<u16>();
            let mut finished = false;

            while !finished && fill_sum > 0 {
                finished = true;
                let indexes: Vec<_> = not_clamped.iter().cloned().collect();
                for j in indexes {
                    let factor = sec_main_factor[j];
                    let size = factor as f32 / fill_sum as f32 * main;
                    let sec_size = sec_main[j];
                    if size < sec_size {
                        finished = false;
                        fill_sum -= factor;
                        not_clamped.remove(&j);
                        sec_main_factor[j] = 0;
                        main -= sec_size
                    }
                }
            }

            for j in 0..nb_sec {
                sec_main[j] = sec_main[j].max(if fill_sum > 0 {
                    sec_main_factor[j] as f32 / fill_sum as f32 * main
                } else {
                    0.
                })
            }
        }

        // ==== Resolve cross ====

        let mut cross = max_cross;

        let mut nodes: Vec<Vec<_>> = self
            .rows
            .iter()
            .map(|vec| vec.iter().map(|_| Node::default()).collect())
            .collect();

        // Compute min cross
        let mut prim_cross = vec![0f32; nb_prim];

        for i in 0..nb_prim {
            for j in 0..nb_sec {
                let (a, b) = axis.pack(i, j);
                let (elt, tree) = {
                    match elts_trees.get_mut(a).and_then(|vec| vec.get_mut(b)) {
                        Some(v) => v,
                        None => continue,
                    }
                };

                let cross_factor = axis.cross(elt.as_widget().size()).fill_factor();

                if cross_factor == 0 {
                    let (max_width, max_height) = axis.pack(sec_main[j], cross);

                    let limits = Limits::new(
                        Size::ZERO,
                        Size {
                            width: max_width,
                            height: max_height,
                        },
                    );

                    let layout = elt.as_widget().layout(tree, renderer, &limits);

                    let size_cross = axis.cross(layout.size());

                    prim_cross[i] = prim_cross[i].max(size_cross);
                    nodes[a][b] = layout;
                }
            }

            cross -= prim_cross[i];
        }

        // Compute main cross

        if cross_length != Shrink {
            let mut not_clamped: HashSet<_> = (0..nb_prim).collect();

            cross = max_cross - cross_total_spacing;

            let mut fill_sum = prim_cross_factor.iter().sum::<u16>();
            let mut finished = false;

            while !finished && fill_sum > 0 {
                finished = true;
                let indexes: Vec<_> = not_clamped.iter().cloned().collect();
                for i in indexes {
                    let factor = prim_cross_factor[i];
                    let size = factor as f32 / fill_sum as f32 * cross;
                    let prim_size = prim_cross[i];
                    if size < prim_size {
                        finished = false;
                        fill_sum -= factor;
                        not_clamped.remove(&i);
                        prim_cross_factor[i] = 0;
                        cross -= prim_size
                    }
                }
            }

            for i in 0..nb_prim {
                prim_cross[i] = prim_cross[i].max(if fill_sum > 0 {
                    prim_cross_factor[i] as f32 / fill_sum as f32 * cross
                } else {
                    0.
                })
            }
        }

        // Compute all nodes
        for i in 0..nb_prim {
            for j in 0..nb_sec {
                let (a, b) = axis.pack(i, j);
                let (elt, tree) = {
                    match elts_trees.get_mut(a).and_then(|vec| vec.get_mut(b)) {
                        Some(v) => v,
                        None => continue,
                    }
                };

                let cross_factor = axis.cross(elt.as_widget().size()).fill_factor();

                if cross_factor != 0 {
                    let max_main = sec_main[j];
                    let max_cross = prim_cross[i];

                    let (max_width, max_height) = axis.pack(max_main, max_cross);

                    let limits = Limits::new(
                        Size::ZERO,
                        Size {
                            width: max_width,
                            height: max_height,
                        },
                    );

                    nodes[a][b] = elt.as_widget().layout(tree, renderer, &limits);
                }
            }
        }

        // Move all the nodes to their correct position
        let (start_x, start_y) = (self.padding.left, self.padding.top);
        let mut x = start_x;
        let mut y = start_y;

        let mut a = 0;
        let mut b = 0;

        for vec_nodes in nodes.iter_mut() {
            for node in vec_nodes.iter_mut() {
                let (i, j) = axis.pack(a, b);

                node.move_to_mut(Point::new(x, y));

                let (width, height) = axis.pack(sec_main[j], prim_cross[i]);

                node.align_mut(
                    self.horizontal_align.into(),
                    self.vertical_align.into(),
                    Size::new(width, height),
                );

                b += 1;
                x += width + self.column_spacing;
            }
            b = 0;
            x = start_x;
            y += match axis {
                Axis::Horizontal => prim_cross[a],
                Axis::Vertical => sec_main[a],
            } + self.row_spacing;
            a += 1;
        }

        let (intrinsic_width, intrinsic_height) = axis.pack(
            sec_main.iter().sum::<f32>() + main_total_spacing,
            prim_cross.iter().sum::<f32>() + cross_total_spacing,
        );

        let size = limits.resolve(
            self.width,
            self.height,
            Size {
                width: intrinsic_width,
                height: intrinsic_height,
            }
            .expand(self.padding),
        );

        Node::with_children(
            size, // size.expand(self.padding),
            nodes.into_iter().flatten().collect(),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            for ((child, state), layout) in self
                .get_elements()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    &clipped_viewport,
                );
            }
        }
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.get_elements()
                .zip(&mut state.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: iced::Event,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> advanced::graphics::core::event::Status {
        self.get_mut_elements()
            .zip(&mut state.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> advanced::mouse::Interaction {
        self.get_elements()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let children = self
            .get_mut_elements()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, state), layout)| {
                child
                    .as_widget_mut()
                    .overlay(state, layout, renderer, translation)
            })
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| advanced::overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message: 'a, Theme: 'a, Renderer: 'a> From<Grid<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn from(value: Grid<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

impl<'a, Message, Theme, Renderer> Grid<'a, Message, Theme, Renderer> {
    fn get_elements(&self) -> impl Iterator<Item = &Element<'a, Message, Theme, Renderer>> {
        self.rows.iter().flatten()
    }

    fn get_mut_elements(
        &mut self,
    ) -> impl Iterator<Item = &mut Element<'a, Message, Theme, Renderer>> {
        self.rows.iter_mut().flatten()
    }
}

/// The main axis of a [Grid].
///
/// See the [Grid::main_axis] method for more info.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// The horizontal axis
    Horizontal,

    /// The vertical axis
    Vertical,
}

impl Axis {
    fn main<T>(&self, size: Size<T>) -> T {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn cross<T>(&self, size: Size<T>) -> T {
        match self {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        }
    }

    fn pack<T>(&self, width: T, height: T) -> (T, T) {
        match self {
            Axis::Horizontal => (width, height),
            Axis::Vertical => (height, width),
        }
    }

    fn size_pack<T>(&self, size: Size<T>) -> (T, T) {
        match self {
            Axis::Horizontal => (size.width, size.height),
            Axis::Vertical => (size.height, size.width),
        }
    }
}

impl Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Axis::Horizontal => "Horizontal",
                Axis::Vertical => "Vertical",
            }
        )
    }
}
