use chord_macro::chord;
use serde::{Deserialize, Serialize};
use std::process::ExitCode;
use tuie::prelude::*;
use tuie::render::border;

mod button;
mod button_item;
mod checkbox;
mod focus_pane;
mod segmented_control;
mod storage;
mod title;

use crate::button::Button;
use crate::button_item::ButtonItem;
use crate::checkbox::Checkbox;
use crate::focus_pane::FocusPane;
use crate::segmented_control::SegmentedControl;
use crate::title::Title;

static PLACEHOLDER_TASK: &str = "What needs to be done ?";

/// Selectable filtre options with display labels.
const FILTERS: &[(Filter, &str)] = &[
    (Filter::All, "All"),
    (Filter::Active, "Active"),
    (Filter::Completed, "Completed"),
];

#[derive(Serialize, Deserialize, Clone)]
struct Task {
    description: String,
    done: bool,
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum Filter {
    All,
    Active,
    Completed,
}

struct TaskList {
    next_task: String,
    filter: Filter,
    tasks: Vec<Task>,
    root: Box<Pane>,
    input_id: WidgetId<Input>,
    button_id: WidgetId<Button>,
    list_id: WidgetId<Pane>,
    items_ids: Vec<WidgetId<Checkbox>>,
    buttons_items_ids: Vec<WidgetId<ButtonItem>>,
    control_id: WidgetId<SegmentedControl>,
}

impl TaskList {
    fn add_task(&mut self) {
        if self.next_task.is_empty() {
            return;
        }
        self.tasks.push(Task {
            description: std::mem::take(&mut self.next_task),
            done: false,
        });
    }

    fn save_task(&mut self) {
        // self.next_task = self.root.get_widget(self.input_id).unwrap().get_string();
        self.next_task = self.root.get_widget(self.input_id).map(Input::get_string).unwrap();

        self.add_task();
        storage::save(&self.tasks);
        self.rebuild_list();

        self.next_task.clear();

        self.root
            .get_widget_mut(self.input_id)
            .unwrap()
            .set_content(String::new());
    }

    fn rebuild_list(&mut self) {
        let tasks = self.tasks.clone();
        let filter = self.filter.clone();

        let mut items_ids = Vec::new();
        let mut buttons_items_ids = Vec::new();

        if let Some(list) = self.get_widget_mut(self.list_id) {
            list.clear();
            for task in &tasks {
                let mut item_id = WidgetId::EMPTY;
                let mut button_item_id = WidgetId::EMPTY;

                if (filter == Filter::Active && !task.done)
                    || (filter == Filter::Completed && task.done)
                    || (filter == Filter::All)
                {
                    let row = Pane::new()
                        .horizontal()
                        .bordered()
                        .gap(2)
                        .border_style(Style::new().fg(Color::grey256(2)))
                        .children([
                            Checkbox::new(Text::new().content(task.description.clone()))
                                .checked_if(task.done)
                                .id(&mut item_id)
                                .flex(3),
                            Pane::new()
                                .child(ButtonItem::new("[ Delete ]", Color::grey256(10)).id(&mut button_item_id))
                                .x_place(Place::End)
                                .flex(1),
                        ]);

                    list.add_child(row);
                }

                items_ids.push(item_id);
                buttons_items_ids.push(button_item_id);
            }
        }

        self.items_ids = items_ids;
        self.buttons_items_ids = buttons_items_ids;
    }
}

impl TaskList {
    fn new(tasks: Vec<Task>) -> Box<Self> {
        let mut input_id = WidgetId::EMPTY;
        let mut button_id = WidgetId::EMPTY;
        let mut list_id = WidgetId::EMPTY;
        let mut control_id = WidgetId::EMPTY;

        let filter_labels: Vec<&str> = FILTERS.iter().map(|(_, l)| *l).collect();

        let root = Pane::new().vertical().gap(2).children([
            Pane::new()
                .horizontal()
                .x_place(Place::Center)
                .children([SegmentedControl::new(&filter_labels).selected(0).id(&mut control_id)]),
            Pane::new()
                .vertical()
                .height(20)
                //.bordered().border(Border::ROUND).border_style(Style::new().fg(Color::CYAN))
                .y_scroll(Scrollbar::Visible)
                .id(&mut list_id),
            Pane::new().horizontal().children([
                FocusPane::new().flex(3).children([Pane::new()
                    .horizontal()
                    .max_height(2)
                    .margin(Spacing::new().horizontal(4))
                    .children([Input::new()
                        .word_wrap()
                        .placeholder(
                            Text::new()
                                .content(PLACEHOLDER_TASK.dim())
                                .style(Style::new().italic().fg(Color::grey256(12))),
                        )
                        .id(&mut input_id)])]),
                Pane::new().flex(1),
                Button::new()
                    .flex(1)
                    .y_align(FlexAlign::Center)
                    .style(Style::new().fg(Color::YELLOW))
                    .children([Text::new()
                        .content("Add Task")
                        .center()
                        .margin(Spacing::new().horizontal(4))])
                    .id(&mut button_id),
            ]),
        ]);

        let mut this = Box::new(Self {
            next_task: String::new(),
            filter: Filter::All,
            tasks,
            root,
            input_id,
            button_id,
            list_id,
            items_ids: Vec::new(),
            buttons_items_ids: Vec::new(),
            control_id,
        });

        this.rebuild_list();
        tuie::runtime::focus_widget(input_id);
        this
    }
}

impl DelegateWidget for TaskList {
    tuie::delegate_widget!(root);

    fn override_on_input(&mut self, queue: &mut InputQueue) -> InputResult {
        let Some(event) = queue.peek() else {
            return InputResult::Rejected;
        };

        match &event.chord {
            chord!(Ctrl + z) => {
                queue.next();
                let _ = tuie::suspend();
                InputResult::Handled
            }
            chord!(Ctrl + (c | q)) => {
                queue.next();
                tuie::quit(0);
                InputResult::Handled
            }

            chord!(Tab) => {
                queue.next();
                tuie::focus_next_tab_order(Sign::Positive);
                InputResult::Handled
            }
            chord!(Shift + Tab) => {
                queue.next();
                tuie::focus_next_tab_order(Sign::Negative);
                InputResult::Handled
            }
            chord!(Enter) => {
                queue.next();
                if self.root.get_widget(self.input_id).unwrap().is_focused()
                    || self.root.get_widget(self.button_id).unwrap().is_focused()
                {
                    self.save_task();
                };
                InputResult::Handled
            }
            _ => InputResult::Rejected,
        }
    }

    fn after_on_event(&mut self, event: &mut WidgetEvent) {
        if let Some(&ChangeEvent(index)) = event.get_by::<ChangeEvent<usize>>(self.control_id) {
            let (filtre, _) = FILTERS[index];
            self.filter = filtre;
            self.rebuild_list();
        }

        if let Some(&ChangeEvent(b)) = event.get::<ChangeEvent<bool>>() {
            for i in 0..self.tasks.len() {
                if event.source == self.items_ids[i] {
                    self.tasks[i].done = b;
                    break;
                }
            }
            storage::save(&self.tasks);
            self.rebuild_list()
        }

        if let Some(&ClickEvent) = event.get::<ClickEvent>() {
            for i in 0..self.tasks.len() {
                if event.source == self.buttons_items_ids[i] {
                    self.tasks.remove(i);
                    break;
                }
            }
            storage::save(&self.tasks);
            self.rebuild_list()
        }

        if event.of_by::<ClickEvent>(self.button_id) {
            self.save_task();
        }
    }
}

fn main() -> std::io::Result<ExitCode> {
    tuie::config::update(|cfg| {
        cfg.expand_tabs = false;
        cfg.hover_events = false;
        cfg.cursor_blink = true;
    });

    tuie::widget::widgets::input::config::update(|cfg| {
        cfg.bindings = ModernBindings::new;
    });

    border::config::update(|cfg| {
        cfg.border = Border::ROUND;
        cfg.selected_style = Style::new().fg(Color::YELLOW);
    });

    let mut tasks = storage::load();

    if tasks.is_empty() {
        tasks = vec![
            Task {
                description: "Buy milk".into(),
                done: false,
            },
            Task {
                description: "Buy eggs".into(),
                done: true,
            },
            Task {
                description: "Buy bread".into(),
                done: false,
            },
        ];
    }

    let root = Pane::new().vertical().border(Border::HIDDEN).gap(1).children([
        Pane::new()
            .vertical()
            .gap(1)
            .flex(1)
            .padding(Spacing::balanced(1))
            .children([Pane::new().x_align(FlexAlign::Center).children([Title::new("T o d o")]) as Box<dyn Widget>]),
        TaskList::new(tasks),
    ]);

    tuie::start_tui(root)
}
