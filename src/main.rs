use std::process::ExitCode;
use tuie::prelude::*;
use tuie::render::border;

mod button;
mod checkbox;
mod focus_pane;
mod global_chords;
mod segmented_control;
mod title;

use crate::button::Button;
use crate::checkbox::Checkbox;
use crate::focus_pane::FocusPane;
use crate::global_chords::GlobalChords;
use crate::segmented_control::SegmentedControl;
use crate::title::Title;

/// Selectable filtre options with display labels.
const FILTERS: &[(Filter, &str)] = &[
    (Filter::All, "All"),
    (Filter::Active, "Active"),
    (Filter::Completed, "Completed"),
];

#[derive(Clone)]
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

    fn rebuild_list(&mut self) {
        let tasks = self.tasks.clone();
        let filter = self.filter.clone();

        let mut items_ids = Vec::new();

        if let Some(list) = self.get_widget_mut(self.list_id) {
            list.clear();
            for task in &tasks {
                let mut item_id = WidgetId::EMPTY;

                if (filter == Filter::Active && !task.done)
                    || (filter == Filter::Completed && task.done)
                    || (filter == Filter::All)
                {
                    let row = Pane::new()
                        .vertical()
                        .bordered()
                        .gap(2)
                        .border_style(Style::new().fg(Color::grey256(2)))
                        .children([Checkbox::new(Text::new().content(task.description.clone()))
                            .checked_if(task.done)
                            .id(&mut item_id)]);
                    list.add_child(row);
                }

                items_ids.push(item_id);
            }
        }

        self.items_ids = items_ids;
    }
}

impl TaskList {
    fn new() -> Box<Self> {
        let mut input_id = WidgetId::EMPTY;
        let mut button_id = WidgetId::EMPTY;
        let mut list_id = WidgetId::EMPTY;

        let mut control_id = WidgetId::EMPTY;

        let next_task = "What needs to be done ?".to_string();

        let tasks = vec![
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

        let filter_labels: Vec<&str> = FILTERS.iter().map(|(_, l)| *l).collect();

        let root = Pane::new().vertical().gap(2).children([
            Pane::new()
                .horizontal()
                .x_place(Place::Center)
                .children([SegmentedControl::new(&filter_labels).selected(0).id(&mut control_id)]),
            Pane::new()
                .vertical()
                .height(20)
                //.bordered()
                //.border(Border::ROUND)
                //.border_style(Style::new().fg(Color::CYAN))
                .y_scroll(Scrollbar::Visible)
                .id(&mut list_id),
            Pane::new().horizontal().children([
                FocusPane::new().flex(3).children([Pane::new()
                    .horizontal()
                    .max_height(2)
                    .margin(Spacing::new().horizontal(4))
                    .children([Input::new()
                        .word_wrap()
                        .content(next_task.clone())
                        // .placeholder(
                        //     Text::new()
                        //         .content(next_task.dim())
                        //         .style(Style::new().italic().fg(Color::grey256(12))),
                        // )
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
            next_task,
            filter: Filter::All,
            tasks,
            root,
            input_id,
            button_id,
            list_id,
            items_ids: Vec::new(),
            control_id,
        });

        this.rebuild_list();

        this
    }
}

impl DelegateWidget for TaskList {
    tuie::delegate_widget!(root);

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
            self.rebuild_list();
        }

        if event.of_by::<ClickEvent>(self.button_id) {
            self.next_task = self.root.get_widget(self.input_id).unwrap().get_string();

            self.add_task();
            self.rebuild_list();

            self.next_task = "What needs to be done ?".to_string();

            self.root
                .get_widget_mut(self.input_id)
                .unwrap()
                .set_content(self.next_task.clone());
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

    let root = Pane::new().vertical().border(Border::HIDDEN).gap(1).children([
        Pane::new()
            .vertical()
            .gap(1)
            .flex(1)
            .padding(Spacing::balanced(1))
            .children([Pane::new().x_align(FlexAlign::Center).children([Title::new("T o d o")]) as Box<dyn Widget>]),
        TaskList::new(),
    ]);

    let root = GlobalChords::new(root);
    tuie::start_tui(root)
}
