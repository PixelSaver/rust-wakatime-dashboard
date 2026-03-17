use eframe::egui;

struct Todo {
    text: String,
    done: bool,
}

struct TodoApp {
    todos: Vec<Todo>,
    new_todo_text: String,
}

impl TodoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            todos: Vec::new(),
            new_todo_text: String::new(),
        }
    }
}

impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My To-Do List");

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let text_input = ui.text_edit_singleline(&mut self.new_todo_text);

                if (ui.button("Add").clicked()
                    || (text_input.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                    && !self.new_todo_text.is_empty()
                {
                    self.todos.push(Todo {
                        text: self.new_todo_text.clone(),
                        done: false,
                    });
                    self.new_todo_text.clear();
                    text_input.request_focus();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            let total = self.todos.len();
            let done_count = self.todos.iter().filter(|t| t.done).count();
            ui.label(format!("{done_count} of {total} tasks completed"));

            ui.add_space(5.0);

            let mut to_remove: Option<usize> = None;

            for (i, todo) in self.todos.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut todo.done, "");

                    if todo.done {
                        ui.label(
                            egui::RichText::new(&todo.text).strikethrough(),
                        );
                    } else {
                        ui.label(&todo.text);
                    }

                    if ui.button("X").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            if let Some(index) = to_remove {
                self.todos.remove(index);
            }

            if self.todos.iter().any(|t| t.done) {
                ui.add_space(10.0);
                if ui.button("Clear completed").clicked() {
                    self.todos.retain(|t| !t.done);
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "To-Do List",
        native_options,
        Box::new(|cc| Ok(Box::new(TodoApp::new(cc)))),
    )
}
