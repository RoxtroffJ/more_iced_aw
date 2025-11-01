use iced::advanced::graphics::core::Element;
use std::ops::{Deref, DerefMut, RangeBounds};

/// A vec containing elements.
///
/// This is mainly usefull to avoid having to call .into() all the time.
///
/// It [`Deref`]s and converts to [`Vec<Element>`]
pub struct ElementVec<'a, Message, Theme, Renderer> {
    /// The inner vec
    pub vec: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> From<Vec<Element<'a, Message, Theme, Renderer>>>
    for ElementVec<'a, Message, Theme, Renderer>
{
    fn from(value: Vec<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self { vec: value }
    }
}

impl<'a, Message, Theme, Renderer> From<ElementVec<'a, Message, Theme, Renderer>>
    for Vec<Element<'a, Message, Theme, Renderer>>
{
    fn from(value: ElementVec<'a, Message, Theme, Renderer>) -> Self {
        value.vec
    }
}

impl<'a, Message, Theme, Renderer> Deref for ElementVec<'a, Message, Theme, Renderer> {
    type Target = Vec<Element<'a, Message, Theme, Renderer>>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<'a, Message, Theme, Renderer> DerefMut for ElementVec<'a, Message, Theme, Renderer> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<'a, Message, Theme, Renderer> ElementVec<'a, Message, Theme, Renderer> {
    /// Create an empty `ElementVec`.
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    /// Create an `ElementVec` with `n` copies of `elem`.
    ///
    /// `E` must be `Clone` so we can convert it multiple times.
    pub fn from_elem<E>(elem: E, n: usize) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>> + Clone,
    {
        let vec = (0..n).map(|_| elem.clone().into()).collect();
        Self { vec }
    }

    /// Push an element that can be converted into an [`Element`].
    pub fn push<E>(&mut self, element: E)
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        self.vec.push(element.into());
    }

    /// Insert an element that can be converted into an [`Element`].
    pub fn insert<E>(&mut self, index: usize, element: E)
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        self.vec.insert(index, element.into());
    }

    /// Extend the vector with elements convertible into [`Element`].
    pub fn extend<E, I>(&mut self, iter: I)
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
        I: IntoIterator<Item = E>,
    {
        for e in iter {
            self.vec.push(e.into());
        }
    }

    /// Replace the given range with elements convertible into [`Element`].
    ///
    /// This mirrors `Vec::splice` but accepts items that implement `Into<Element>`.
    pub fn splice<'b, R, I, E>(
        &'b mut self,
        range: R,
        replace_with: I,
    ) -> std::vec::Splice<'b, std::vec::IntoIter<Element<'a, Message, Theme, Renderer>>>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = E>,
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        let converted: Vec<_> = replace_with.into_iter().map(|e| e.into()).collect();
        self.vec.splice(range, converted)
    }
}

impl<'a, Message, Theme, Renderer, E> Extend<E> for ElementVec<'a, Message, Theme, Renderer>
where
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = E>,
    {
        for e in iter {
            self.vec.push(e.into());
        }
    }
}

impl<'a, Message, Theme, Renderer, E> FromIterator<E> for ElementVec<'a, Message, Theme, Renderer>
where
    E: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        let vec = iter.into_iter().map(|e| e.into()).collect();
        Self { vec }
    }
}

impl<'a, Message, Theme, Renderer> Default for ElementVec<'a, Message, Theme, Renderer> {
    fn default() -> Self {
        Self { vec: Vec::new() }
    }
}

impl<'a, Message, Theme, Renderer> IntoIterator for ElementVec<'a, Message, Theme, Renderer> {
    type Item = Element<'a, Message, Theme, Renderer>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a, 'b, Message, Theme, Renderer> IntoIterator
    for &'b ElementVec<'a, Message, Theme, Renderer>
{
    type Item = &'b Element<'a, Message, Theme, Renderer>;
    type IntoIter = std::slice::Iter<'b, Element<'a, Message, Theme, Renderer>>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl<'a, 'b, Message, Theme, Renderer> IntoIterator
    for &'b mut ElementVec<'a, Message, Theme, Renderer>
{
    type Item = &'b mut Element<'a, Message, Theme, Renderer>;
    type IntoIter = std::slice::IterMut<'b, Element<'a, Message, Theme, Renderer>>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter_mut()
    }
}

#[macro_export]
/// Same as [`vec`](std::vec!), but builds a [`ElementVec`].
///
/// This means that the elements provided can just implement [Into<Element>].
macro_rules! element_vec {
    () => ($crate::helpers::ElementVec::new());
    ($elem:expr; $n:expr) => ($crate::helpers::ElementVec::from_elem($elem, $n));
    ($($x:expr),+ $(,)?) => ($crate::helpers::ElementVec::from(
        <[_]>::into_vec(
            // Using the intrinsic produces a dramatic improvement in stack usage for
            // unoptimized programs using this code path to construct large Vecs.
            std::boxed::box_new([$(iced::advanced::graphics::core::Element::from($x)),+])
        )
    ));
}
