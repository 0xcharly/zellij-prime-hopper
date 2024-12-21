use crate::{
    core::{InternalError, PluginError, PluginUpdateLoop},
    matcher::{Choice, Match},
};

use std::collections::BTreeSet;
use std::{
    path::PathBuf,
    rc::{Rc, Weak},
};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher as _;

#[derive(Default, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct PathEntry {
    repr: Option<PathBuf>,
    path: PathBuf,
}

impl PathEntry {
    pub(super) fn new(repr: PathBuf, path: PathBuf) -> Self {
        Self {
            repr: Some(repr),
            path,
        }
    }

    pub(super) fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl From<PathBuf> for PathEntry {
    fn from(path: PathBuf) -> Self {
        Self { repr: None, path }
    }
}

impl Choice for PathEntry {
    fn repr<'c>(&'c self) -> &'c str {
        self.repr
            .as_ref()
            .unwrap_or(&self.path)
            .to_str()
            .unwrap_or_else(|| todo!("Replace non-UTF8 characters with �"))
    }
}

struct FuzzyMatcher {
    matcher: SkimMatcherV2,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self {
            matcher: SkimMatcherV2::default().use_cache(true),
        }
    }
}

impl FuzzyMatcher {
    fn apply<C: Choice>(&self, input: &str, choices: &BTreeSet<Rc<C>>) -> Vec<Match<C>> {
        choices
            .iter()
            .filter_map(|choice| {
                self.matcher
                    .fuzzy_indices(choice.repr(), input)
                    .map(|(_score, indices)| Match {
                        indices,
                        choice: Rc::downgrade(choice),
                    })
            })
            .collect()
    }
}

/// The plugin context holds volatile state such as non-fatal errors that should be reported to the
/// user via the UI.
#[derive(Default)]
pub(crate) struct FuzzySearchContext {
    /// The user input query.
    user_input: String,

    /// The unfiltered list of elements to run the fuzzy matcher on.
    choices: BTreeSet<Rc<PathEntry>>,

    /// List of choice indices from [choices].
    matches: Vec<Match<PathEntry>>,

    /// The index of the currently selected choice relative to [filtered_choices].
    selected_index: usize,

    // TODO: Add support for tracking the selected match: i.e. instead of only tracking the
    // selected index, also track the selected match in case it appears in successive results.
    //
    // For example, given the current set of results and selection cursor:
    //
    // ```
    //     nix-config
    //     nix-config-ghostty
    //   > nix-config-nvim
    //     nix-config-manager
    // ```
    //
    // And assuming the next user input changes the results to:
    //
    // ```
    //     nix-config-ghostty
    //   > nix-config-nvim
    //     nix-config-manager
    // ```
    //
    // Keeping track of the selected match would allow us to move the cursor to the second entry
    // (the one the user manually selected already) instead of leaving it on the third one.
    #[allow(dead_code)]
    selected_match: Option<Weak<PathEntry>>,

    /// Non-fatal errors raised during plugin execution. While non-fatal, some errors may not be
    /// recoverable.
    errors: Vec<PluginError>,

    /// Matches the list of repositories against the user input. Keeps track of the user input.
    matcher: FuzzyMatcher,
}

impl FuzzySearchContext {
    pub(super) fn user_input(&self) -> &str {
        &self.user_input
    }

    pub(super) fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub(super) fn selected_match(&mut self) -> Option<Rc<PathEntry>> {
        // TODO: Use self.selected_match which should keep track of the selected item in a more
        // stable fashion.
        //self.selected_match
        //    .as_ref()
        //    .and_then(Weak::<PathEntry>::upgrade)

        // NOTE: fallback on self.selected_index for now.
        let Some(selected_match) = self.matches.get(self.selected_index) else {
            self.log_internal_error(InternalError::SelectionIndexOutOfBounds(
                self.selected_index,
            ));
            return None;
        };
        selected_match.choice.upgrade()
    }

    pub(super) fn on_user_input(&mut self, ch: char) -> PluginUpdateLoop {
        self.clear_errors();
        self.user_input.push(ch);

        self.invalidate_matches();

        // Force update since the user input changed (even if the list of results may not have as a
        // result, the matched characters will have changed).
        PluginUpdateLoop::MarkDirty
    }

    pub(super) fn remove_trailing_char(&mut self) -> PluginUpdateLoop {
        let update = self.clear_errors();

        if self.user_input.pop().is_some() {
            self.invalidate_matches();
            return PluginUpdateLoop::MarkDirty;
        }

        update
    }

    pub(super) fn clear_user_input(&mut self) -> PluginUpdateLoop {
        let update = self.clear_errors();

        if self.user_input.is_empty() {
            return update;
        }

        self.user_input.clear();
        self.invalidate_matches();
        PluginUpdateLoop::MarkDirty
    }

    pub(super) fn select_up(&mut self) -> PluginUpdateLoop {
        let update = self.clear_errors();
        let previous_index = self.selected_index;
        self.selected_index = self
            .selected_index
            .saturating_sub(1)
            .clamp(0, self.matches.len().saturating_sub(1));
        update | PluginUpdateLoop::from(previous_index != self.selected_index)
    }

    pub(super) fn select_down(&mut self) -> PluginUpdateLoop {
        let update = self.clear_errors();
        let previous_index = self.selected_index;
        self.selected_index = self
            .selected_index
            .saturating_add(1)
            .clamp(0, self.matches.len().saturating_sub(1));
        update | PluginUpdateLoop::from(previous_index != self.selected_index)
    }

    #[cfg(feature = "zellij_fallback_fs_api")]
    pub(super) fn add_choice(&mut self, choice: PathEntry) -> PluginUpdateLoop {
        self.choices.insert(choice.into());
        self.invalidate_matches();

        PluginUpdateLoop::MarkDirty
    }

    pub(super) fn add_choices(
        &mut self,
        choices: impl Iterator<Item = PathEntry>,
    ) -> PluginUpdateLoop {
        self.choices
            .extend(choices.map(Into::<Rc<PathEntry>>::into));
        self.invalidate_matches();

        PluginUpdateLoop::MarkDirty
    }

    pub(super) fn choice_count(&self) -> usize {
        self.choices.len()
    }

    pub(super) fn match_count(&self) -> usize {
        self.matches.len()
    }

    pub(super) fn matches(&self) -> impl Iterator<Item = &Match<PathEntry>> {
        self.matches.iter()
    }

    pub(super) fn log_error(&mut self, error: PluginError) -> PluginUpdateLoop {
        self.errors.push(error);
        PluginUpdateLoop::MarkDirty
    }

    fn log_internal_error(&mut self, error: InternalError) {
        self.errors.push(PluginError::UnexpectedError(error.into()));
    }

    pub(super) fn clear_errors(&mut self) -> PluginUpdateLoop {
        self.errors.clear();
        PluginUpdateLoop::MarkDirty
    }

    pub(super) fn errors(&self) -> &Vec<PluginError> {
        &self.errors
    }

    fn invalidate_matches(&mut self) {
        self.matches = self.matcher.apply(&self.user_input, &self.choices);

        // Clamp selected_index.
        self.selected_index = self
            .selected_index
            .clamp(0, self.matches.len().saturating_sub(1));

        // TODO: update self.selected_match
    }
}
