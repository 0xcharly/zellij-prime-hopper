use crate::fuzzy_search_context::FuzzySearchContext;

use styles::Styles;

mod frame;
mod renderer;
mod styles;

pub const PANE_TITLE: &'static str = "Select a directory:";

#[derive(Default)]
pub(crate) struct Renderer {
    styles: Styles,
}

/// Represents a plugin UI frame of size [rows]×[cols].
///
/// Implements the [std::fmt::Display] trait to easily render it via Zellij's API.
pub(crate) struct Frame<'ui> {
    rows: usize,
    cols: usize,
    context: &'ui FuzzySearchContext,
    styles: &'ui Styles,
}
