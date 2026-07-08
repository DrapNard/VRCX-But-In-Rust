use crate::models::users::User;
mod api;
mod client;
mod error;
mod models;
mod session;
use iced::widget::{Column, button, column, text};

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Clone)]
enum Message {
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }
    fn view(&self) -> Column<Message> {
        column![
            button("+").on_press(Message::Increment),
            text(self.value),
            button("-").on_press(Message::Decrement),
        ]
    }
}

pub fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}
