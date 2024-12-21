use super::{Frame, Renderer};
use crate::fuzzy_search_context::FuzzySearchContext;

impl Renderer {
    pub fn next_frame<'ui>(
        &'ui self,
        rows: usize,
        cols: usize,
        context: &'ui FuzzySearchContext,
    ) -> Frame<'ui> {
        Frame {
            rows,
            cols,
            context,
            styles: &self.styles,
        }
    }
}
