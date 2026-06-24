//! Two-state checkbox widget.
//!
use chord_macro::chord;
use std::cell::Cell;
use tuie::{field, prelude::*};

thread_local! {
    static ACCENT: Cell<Style> = Cell::new(Style::new().fg(Color::BLUE));
}

/// Two-state checkbox that emits [`ChangeEvent`].
pub struct Checkbox {
    root: Box<Pane>,
    indicator_id: WidgetId<Text>,
    checked: bool,
}

impl Checkbox {
    fn toggle(&mut self) {
        self.set_checked(!self.checked);
        tuie::emit(self.get_id(), ChangeEvent(self.checked));
    }

    fn sync_indicator(&mut self) {
        let icon = if self.checked { "[x]" } else { "[ ]" };
        if let Some(text) = self.root.get_widget_mut(self.indicator_id) {
            text.set_content(icon);
        }
    }

    field!(checked: bool; sync_indicator);
}

impl DelegateWidget for Checkbox {
    tuie::delegate_widget!(root);

    fn override_is_focusable(&self) -> bool {
        true
    }

    fn after_on_state_change(&mut self, state: WidgetState) {
        let accent = ACCENT.with(|c| c.get());
        let style = match state {
            WidgetState::Focused | WidgetState::FocusedHover => accent.bold(),
            WidgetState::Active => accent,
            _ => Style::new(),
        };
        self.root.set_style(style);
    }

    fn override_on_input(&mut self, queue: &mut InputQueue) -> InputResult {
        let Some(event) = queue.next() else {
            return InputResult::Rejected;
        };
        match &event.chord {
            chord!(Enter | Space) => {
                self.toggle();
            }
            chord!(LeftClick) => {
                tuie::focus_widget(self.get_id());
            }
            chord!(LeftRelease) => {
                let size = self.get_rect_size();
                if Axis2D::all(|a| event.cell()[a] >= 0 && event.cell()[a] < size[a] as i32) {
                    self.toggle();
                }
            }
            _ => return InputResult::Rejected,
        }
        InputResult::Handled
    }
}

impl Checkbox {
    /// Creates an unchecked checkbox with the given label.
    pub fn new(label: Box<Text>) -> Box<Self> {
        let mut indicator_id = WidgetId::EMPTY;
        let root = Pane::new()
            .horizontal()
            .gap(1)
            .children([Text::new().content("[ ]").id(&mut indicator_id), label]);
        Box::new(Self {
            root,
            indicator_id,
            checked: false,
        })
    }
}
