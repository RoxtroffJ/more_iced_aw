#![warn(missing_docs)]

//! Adds additionnal iced widgets. Some are inspired by iced_aw.
//! 
//! All widgets that have a state support serialization and deserialization with serde if the feature `serde` is enabled.

pub mod parsed_input;
pub mod grid;
pub mod helpers;