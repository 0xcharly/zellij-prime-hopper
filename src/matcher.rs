use std::rc::Weak;

/// A trait allowing arbitrary data to be matched against the user input.
pub(super) trait Choice {
    fn repr(&self) -> &str;
}

/// A match against the user input.
/// Returned by the matcher and used by the renderer to display the list of matches.
pub(super) struct Match<C: Choice> {
    /// A weak reference to the entry.
    // TODO: Does it make sense to keep a `Weak<C>` here? We're only using `Rc<C>` to keep multiple
    // references on the source-of-truth instances stored in the `FuzzyFinderContext`, so juggling
    // between `Weak` and `Rc` seems unnecessary since we expect `Weak`s to always be promotable to
    // `Rc`s.
    pub choice: Weak<C>,

    /// The list of indices in [Choice::repr()] that matched against the user input.
    /// Used by the renderer to highlight matches.
    pub indices: Vec<usize>,
}
