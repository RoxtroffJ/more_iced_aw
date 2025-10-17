//! Similar to iced_aw's `TypedInput` widget, except this one
//! clearly indicates when the text does not match the actual value.
//!
//! It is also not stateless, which allows to modify the value by other means than
//! interacting with this widget, while the state makes sure to keep track of whether
//! the text matches the value or not.
//!
//! # Example
//!
//! ```
//! use iced::{self, Element, widget::{text_input, row, text, column}, color, alignment::Vertical};
//! use more_iced_aw::parsed_input::*;
//!
//! #[derive(Default)]
//! struct App {
//!     content: Content<i8, std::num::ParseIntError>,
//!     msg: String
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Input(Parsed<i8, std::num::ParseIntError>),
//!     Paste(Parsed<i8, std::num::ParseIntError>),
//!     Submit
//! }
//!
//! impl App {
//!     fn update(&mut self, message: Message) {
//!         self.msg = format!("{message:?}");
//!         match message {
//!             Message::Input(parsed) => self.content.update(parsed),
//!             Message::Paste(parsed) => self.content.update(parsed),
//!             Message::Submit => if self.content.is_valid() {
//!                 let mut val = self.content.borrow_mut();
//!                 *val += 1
//!             }
//!         }
//!     }
//!
//!     fn view(&self) -> Element<'_, Message> {
//!         let input = ParsedInput::new("Type an integer", &self.content)
//!         .style(color_on_err(text_input::default, color!(0xff0000, 0.2)))
//!         .on_input(Message::Input)
//!         .on_paste(Message::Paste)
//!         .on_submit(Message::Submit);
//!         
//!         let row = row![input]
//!         .push_maybe(
//!             self
//!             .content
//!             .get_error()
//!             .as_ref()
//!             .map(|err| text(err.to_string()))
//!          )
//!         .align_y(Vertical::Center)
//!         .spacing(10);
//!         
//!         column![row, text(&self.msg)].spacing(20).into()
//!     }
//! }
//!
//! fn main() -> iced::Result {
//!     iced::run("Parsed Input", App::update, App::view)
//! }
//! ```

use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use iced::{
    Background, Color, Gradient, Length, Padding, Pixels,
    advanced::{Shell, Widget, graphics::core::Element, text},
    alignment,
    gradient::{ColorStop, Linear},
    widget::{
        TextInput,
        text_input::{self, Icon, Id, Status, Style, StyleFn},
    },
};

use crate::helpers::filter_color;

/// The content of the [`ParsedInput`] for a value of type `T` and parsing errors of type `E`.
///
/// It implements [`Deref`] into `T`, which allows you to access the inner value.
/// To modify `T`, you must first call [`borrow_mut`](Content::borrow_mut)
/// and the outputed [`BorrowMut`] will implement [`DerefMut`] into `T` (see this [`example`](crate::parsed_input))
/// 
/// # Assumptions
/// 
/// For a [`ParsedInput`] build on this [`Content`] to work as intendeed, 
/// it is mendatory that for all `value: T`,
/// `value.to_string().parse() == Ok(value)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content<T, E> {
    value: T,
    string: String,
    error: Option<E>,
}

impl<T, E> Content<T, E> {
    /// Creates a new content.
    pub fn new(value: T) -> Self
    where
        T: ToString,
    {
        let string = value.to_string();
        Self {
            value,
            string,
            error: None,
        }
    }

    /// Mutably borrows the inner value (`T`), to then be able to modify it.
    ///
    /// The returned [`BorrowMut`] implements [`DerefMut<Target: T>`]. 
    /// When dropped, it will set the string of `self` (that is displayed
    /// in the [`ParsedInput`]) to `value.to_string()`.
    pub fn borrow_mut(&mut self) -> BorrowMut<'_, T, E>
    where
        T: ToString,
    {
        BorrowMut { content: self }
    }

    /// Indicates if the value corresponds to the string.
    pub fn is_valid(&self) -> bool {
        self.error.is_none()
    }

    /// Returns the parsing error if there is one.
    pub fn get_error(&self) -> &Option<E> {
        &self.error
    }

    /// Updates the content with the given [`Parsed`].
    /// 
    /// See this [example](crate::parsed_input) for recommended usage.
    pub fn update(&mut self, parsed: Parsed<T, E>) {
        self.string = parsed.string;
        match parsed.parsed {
            Ok(val) => {
                self.error = None;
                self.value = val
            }
            Err(err) => self.error = Some(err),
        }
    }
}

/// An inner message that will be produced by the inner [`TextInput`].
#[derive(Debug, Clone)]
enum InnerMessage {
    /// The user inputed a string.
    Input(String),
    /// The user pasted a string.
    Paste(String),
    /// The user submited.
    Submit,
}

/// A string and parser result.
///
/// You can't modify it unless you deconstruct it and rebuild it.
/// It is used in the messages produced by a [`ParsedInput`] and
/// allows to update a [`Content`]. 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed<T, E> {
    string: String,
    parsed: Result<T, E>,
}

impl<T, E> Parsed<T, E> {
    /// Builds a [`Parsed`] from a [`String`].
    pub fn from_string(str: &str) -> Self
    where
        T: FromStr<Err = E>,
    {
        Self {
            string: str.to_string(),
            parsed: str.parse(),
        }
    }

    /// Builds a [`Parsed`] from a value.
    pub fn from_value(value: T) -> Self
    where
        T: ToString,
    {
        Self {
            string: value.to_string(),
            parsed: Ok(value),
        }
    }

    /// Gets the values contained in the [`Parsed`].
    pub fn take(self) -> (String, Result<T, E>) {
        (self.string, self.parsed)
    }

    /// Returns a reference to the contained [`String`].
    pub fn get_string(&self) -> &String {
        &self.string
    }

    /// Returns a reference to the contained parsed [`Result`]
    pub fn get_result(&self) -> &Result<T, E> {
        &self.parsed
    }
}

/// The [`ParsedInput`] widget.
///
/// It is fundamentally a [`TextInput`] and therefore implements the same methods.
pub struct ParsedInput<'a, T, E, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::text::Renderer,
    Theme: text_input::Catalog,
{
    content: &'a Content<T, E>,
    text_input: TextInput<'a, InnerMessage, Theme, Renderer>,

    on_input: Option<Box<dyn Fn(Parsed<T, E>) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(Parsed<T, E>) -> Message + 'a>>,
    on_submit: Option<Message>,
}

impl<'a, T, E, Message, Theme, Renderer> ParsedInput<'a, T, E, Message, Theme, Renderer>
where
    T: Clone,
    E: Clone,
    Renderer: iced::advanced::text::Renderer,
    Theme: text_input::Catalog + 'a,
{
    /// Creates a new [`ParsedInput`] from a [`Content`].
    pub fn new(placeholder: &str, content: &'a Content<T, E>) -> Self {
        Self {
            content,
            text_input: TextInput::new(placeholder, &content.string),
            on_input: None,
            on_paste: None,
            on_submit: None,
        }
    }

    /// Sets the [`Id`] of the underlying [`TextInput`].
    pub fn id(self, id: impl Into<Id>) -> Self {
        Self {
            text_input: self.text_input.id(id),
            ..self
        }
    }

    /// Converts the underlying [`TextInput`] into a secure password input.
    pub fn secure(self, is_secure: bool) -> Self {
        Self {
            text_input: self.text_input.secure(is_secure),
            ..self
        }
    }

    /// Sets the message that should be produced when some text is typed into the [`ParsedInput`].
    ///
    /// If this method is not called, the [`ParsedInput`] will be disabled.
    pub fn on_input(self, on_input: impl Fn(Parsed<T, E>) -> Message + 'a) -> Self {
        Self {
            text_input: self.text_input.on_input(InnerMessage::Input),
            on_input: Some(Box::new(on_input)),
            ..self
        }
    }

    /// Sets the message that should be produced when some text is typed into the [`ParsedInput`], if [`Some`].
    ///
    /// If this method is not called, the [`ParsedInput`] will be disabled.
    pub fn on_input_maybe(self, on_input: Option<impl Fn(Parsed<T, E>) -> Message + 'a>) -> Self {
        match on_input {
            Some(on_input) => self.on_input(on_input),
            None => self,
        }
    }

    /// Sets the message that should be produced when the [`ParsedInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, on_submit: Message) -> Self {
        self.text_input = self.text_input.on_submit(InnerMessage::Submit);
        self.on_submit = Some(on_submit);
        self
    }

    /// Sets the message that should be produced when the [`ParsedInput`] is
    /// focused and the enter key is pressed, if `Some`.
    pub fn on_submit_maybe(self, on_submit: Option<Message>) -> Self {
        match on_submit {
            Some(on_submit) => self.on_submit(on_submit),
            None => todo!(),
        }
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`ParsedInput`].
    pub fn on_paste(mut self, on_paste: impl Fn(Parsed<T, E>) -> Message + 'a) -> Self {
        self.text_input = self.text_input.on_paste(InnerMessage::Paste);
        self.on_paste = Some(Box::new(on_paste));
        self
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`ParsedInput`], if `Some`.
    pub fn on_paste_maybe(self, on_paste: Option<impl Fn(Parsed<T, E>) -> Message + 'a>) -> Self {
        match on_paste {
            Some(on_paste) => self.on_paste(on_paste),
            None => self,
        }
    }

    /// Sets the [`Font`] of the [`ParsedInput`].
    ///
    /// [`Font`]: text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.text_input = self.text_input.font(font);
        self
    }

    /// Sets the [`Icon`] of the [`ParsedInput`].
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.text_input = self.text_input.icon(icon);
        self
    }

    /// Sets the width of the [`ParsedInput`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.text_input = self.text_input.width(width);
        self
    }

    /// Sets the [`Padding`] of the [`ParsedInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.text_input = self.text_input.padding(padding);
        self
    }

    /// Sets the text size of the [`ParsedInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_input = self.text_input.size(size);
        self
    }

    /// Sets the [`text::LineHeight`] of the [`ParsedInput`].
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_input = self.text_input.line_height(line_height);
        self
    }

    /// Sets the horizontal alignment of the [`ParsedInput`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.text_input = self.text_input.align_x(alignment);
        self
    }

    /// Sets the style of the [`ParsedInput`].
    ///
    /// Compared to a style function of a [`TextInput`], this one also takes
    /// an additionnal bool which indicates if the string matched the value (true)
    /// or not (false).
    pub fn style(mut self, style: impl Fn(&Theme, Status, bool) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.text_input = if self.content.is_valid() {
            self.text_input.style(move |t, s| style(t, s, true))
        } else {
            self.text_input.style(move |t, s| style(t, s, false))
        };
        self
    }

    /// Sets the style class of the [`ParsedInput`].
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.text_input = self.text_input.class(class);
        self
    }
}

impl<'a, T: FromStr<Err = E>, E, Message: Clone, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ParsedInput<'a, T, E, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
    Theme: text_input::Catalog,
{
    fn state(&self) -> iced::advanced::widget::tree::State {
        self.text_input.state()
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.text_input.tag()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.text_input.diff(tree);
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.text_input.children()
    }

    fn size(&self) -> iced::Size<Length> {
        <TextInput<'_, _, _, _> as Widget<_, _, _>>::size(&self.text_input)
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        <TextInput<'_, _, _, _> as Widget<_, _, _>>::layout(
            &self.text_input,
            tree,
            renderer,
            limits,
        )
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        <TextInput<'_, _, _, _> as Widget<_, _, _>>::draw(
            &self.text_input,
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &self,
        state: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.text_input.operate(state, layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        state: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        let mut messages = Vec::new();
        let mut sub_shell = Shell::new(&mut messages);
        let status = self.text_input.on_event(
            state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut sub_shell,
            viewport,
        );

        shell.merge(sub_shell, |inner| match inner {
            InnerMessage::Input(str) => self
                .on_input
                .as_ref()
                .map(|f| f(Parsed::from_string(&str)))
                .expect("Should have on_input msg"),
            InnerMessage::Paste(str) => self
                .on_paste
                .as_ref()
                .map(|f| f(Parsed::from_string(&str)))
                .expect("Should have on_paste msg"),
            InnerMessage::Submit => self
                .on_submit
                .as_ref()
                .cloned()
                .expect("Should have submit msg"),
        });

        status
    }

    fn mouse_interaction(
        &self,
        state: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.text_input
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }

    fn size_hint(&self) -> iced::Size<Length> {
        self.text_input.size_hint()
    }
}

impl<'a, T: FromStr<Err = E>, E, Message: Clone + 'a, Theme: 'a, Renderer: 'a>
    From<ParsedInput<'a, T, E, Message, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
    Theme: text_input::Catalog,
{
    fn from(value: ParsedInput<'a, T, E, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

/// A mutable borrow of the inner value of a [`Content`].
/// 
/// It allows to change said value without having the value
/// and the string of the [`Content`] going out of sync.
pub struct BorrowMut<'a, T: ToString, E> {
    content: &'a mut Content<T, E>,
}

/// Returns a [`text_input::Style`] and applies a color to it's background when the [`ParsedInput`] has an invalid [`String`].
pub fn color_on_err<Theme>(
    style: impl Fn(&Theme, Status) -> Style,
    color: Color,
) -> impl Fn(&Theme, Status, bool) -> Style {
    move |theme, status, valid| {
        let style = style(theme, status);
        if valid {
            style
        } else {
            let background = match style.background {
                iced::Background::Color(c) => Background::Color(filter_color(c, color)),
                iced::Background::Gradient(gradient) => match gradient {
                    iced::Gradient::Linear(linear) => {
                        let new_stops = linear.stops.map(|x| {
                            x.map(|stop| ColorStop {
                                color: filter_color(stop.color, color),
                                ..stop
                            })
                        });

                        Background::Gradient(Gradient::Linear(Linear {
                            stops: new_stops,
                            ..linear
                        }))
                    }
                },
            };

            text_input::Style {
                background,
                ..style
            }
        }
    }
}

impl<T: Default + ToString, E> Default for Content<T, E> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T, E> AsRef<T> for Content<T, E> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T, E> Borrow<T> for Content<T, E> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T, E> Deref for Content<T, E> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T: ToString, E> AsRef<T> for BorrowMut<'a, T, E> {
    fn as_ref(&self) -> &T {
        &**self
    }
}
impl<'a, T: ToString, E> AsMut<T> for BorrowMut<'a, T, E> {
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}
impl<'a, T: ToString, E> Borrow<T> for BorrowMut<'a, T, E> {
    fn borrow(&self) -> &T {
        &**self
    }
}
impl<'a, T: ToString, E> std::borrow::BorrowMut<T> for BorrowMut<'a, T, E> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<'a, T: ToString, E> Deref for BorrowMut<'a, T, E> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content.value
    }
}

impl<'a, T: ToString, E> DerefMut for BorrowMut<'a, T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content.value
    }
}

impl<'a, T: ToString, E> Drop for BorrowMut<'a, T, E> {
    fn drop(&mut self) {
        self.content.string = self.content.value.to_string();
        self.content.error = None;
    }
}
