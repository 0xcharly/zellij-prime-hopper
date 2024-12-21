use std::collections::BTreeSet;
use std::rc::{Rc, Weak};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher as _;

pub trait Choice {
    fn repr(&self) -> &str;
}

pub(crate) struct Match<C: Choice> {
    pub indices: Vec<usize>,
    pub choice: Weak<C>,
}

pub(crate) struct FuzzyMatcher {
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
    pub(super) fn apply<C: Choice>(&self, input: &str, choices: &BTreeSet<Rc<C>>) -> Vec<Match<C>> {
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
