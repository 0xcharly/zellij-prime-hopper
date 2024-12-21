# Behavior

- [ ] Prevent new windows of the plugin opening when one already exists (we have
  to use `launch_new` to force update $CWD, so we might have to deal with window
  management manually).

- [ ] Add support for vertical scrolling in the list of results.

- [ ] Investigate whether the fs fallback API is still required (since the
  canonical bug https://github.com/zellij-org/zellij/issues/2556 is already
  closed), and deprecate/remove it if possible.

- [ ] Add support for tracking the current selection when the user updates the
  filter and the currently selected item (previously at index `i`) is still
  visible (but now at index `j`).
