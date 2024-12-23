use super::{
    styles::{ControlBar, ControlSegment},
    Frame,
};
use std::fmt::{Display, Formatter, Result};

const SEARCH_PREFIX: &'static str = ">";

/// Takes into account the following constantly visble lines:
///   - At the top, the first 2 lines:
///     - User input
///     - Separator
///   - At the bottom, the last 2 lines:
///     - Tips
///     - Status line
const CHROME_LINE_COUNT: usize = 4;

const CONTROL_BAR: ControlBar = ControlBar {
    segments: [
        ControlSegment {
            control: "↓↑",
            short_label: "Navigate",
            full_label: "Navigate between entries",
        },
        ControlSegment {
            control: "ENTER",
            short_label: "Select",
            full_label: "Select entry",
        },
        ControlSegment {
            control: "ESC",
            short_label: "Clear",
            full_label: "Clear input",
        },
    ],
};

impl<'ui> Frame<'ui> {
    fn fmt_pane_too_small(&self, f: &mut Formatter<'_>) -> Result {
        self.styles.fmt_pane_too_small(f)
    }

    fn fmt_user_input(&self, f: &mut Formatter<'_>) -> Result {
        self.styles
            .fmt_user_input(f, &SEARCH_PREFIX, self.context.user_input())
    }

    fn fmt_user_input_divider(&self, f: &mut Formatter<'_>) -> Result {
        self.styles.fmt_user_input_divider(
            f,
            self.context.match_count(),
            self.context.choice_count(),
            self.cols,
        )
    }

    fn fmt_matched_results(&self, f: &mut Formatter<'_>) -> Result {
        self.styles.fmt_matched_results(
            f,
            self.context.matches(),
            self.context.selected_index(),
            self.rows.saturating_sub(CHROME_LINE_COUNT),
            self.cols,
        )
    }

    fn fmt_spacer(&self, f: &mut Formatter<'_>) -> Result {
        for _ in 0..self
            .rows
            .saturating_sub(self.context.match_count() + CHROME_LINE_COUNT)
        {
            writeln!(f)?;
        }

        Ok(())
    }

    fn fmt_control_bar(&self, f: &mut Formatter<'_>) -> Result {
        self.styles.fmt_control_bar(f, &CONTROL_BAR, self.cols)
    }

    /// Prints errors, if any.
    /// Since this is the last line, skip the final newline.
    fn fmt_status_bar(&self, f: &mut Formatter<'_>) -> Result {
        self.styles.fmt_status_bar(f, &self.context)
    }
}

impl Display for Frame<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Bail immediately if we don't have the space to render the bare minimum UI, which
        // consists of the chrome and at least 1 row of results.
        if self.rows < CHROME_LINE_COUNT + 1 {
            return self.fmt_pane_too_small(f);
        }

        // Header.
        self.fmt_user_input(f)?;
        self.fmt_user_input_divider(f)?;

        // Body.
        self.fmt_matched_results(f)?;

        // Spacer: if there's less results than available lines for display, fill up the pane with
        // padding down to the footer.
        self.fmt_spacer(f)?;

        // Footer.
        self.fmt_control_bar(f)?;
        self.fmt_status_bar(f)?;

        Ok(())
    }
}
