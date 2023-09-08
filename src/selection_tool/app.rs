use crate::selection_tool::rectangle::Rectangle;
use iced::{
    widget::{column, container},
    Alignment, Command, Element, Length, Point, Size,
};
use iced_wgpu::Renderer;
use iced_winit::runtime::{program::State, Debug, Program};
use log::info;

use super::theme::Theme;

pub mod state;

#[derive(Debug, Clone)]
pub enum Message {
    OnMousePressed,
    OnMouseMoved(Point),
    OnMouseReleased,
}

pub struct App {
    width: f32,
    height: f32,
    pressed: bool,
    released: bool,
    cursor_pressed_position: Point,
    cursor_released_position: Point,
}

impl App {
    pub fn program_state(
        size: Size<f32>,
        renderer: &mut <Self as Program>::Renderer,
        debug: &mut Debug,
    ) -> State<Self> {
        State::new(Default::default(), size, renderer, debug)
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            width: 0f32,
            height: 0f32,
            pressed: false,
            released: false,
            cursor_pressed_position: Point { x: 0.0, y: 0.0 },
            cursor_released_position: Point { x: 0.0, y: 0.0 },
        }
    }
}

impl Program for App {
    type Message = Message;
    type Renderer = Renderer<Theme>;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OnMousePressed => {
                info!("Mouse pressed");
                self.width = 0f32;
                self.height = 0f32;
                self.pressed = true;
                self.released = false;
                Command::none()
            }

            Message::OnMouseMoved(point) => {
                if self.pressed && !self.released {
                    self.width = point.x - self.cursor_pressed_position.x;
                    self.height = point.y - self.cursor_pressed_position.y;
                    self.cursor_released_position = point;
                } else if !self.released {
                    self.cursor_pressed_position = point;
                }
                Command::none()
            }

            Message::OnMouseReleased => {
                info!("Mouse released");
                self.pressed = false;
                self.released = true;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message, Renderer<Theme>> {
        let content = column![Rectangle::new(self.width, self.height)]
        .padding([self.cursor_pressed_position.y, self.cursor_pressed_position.x])
        .spacing(0)
        .align_items(Alignment::Start);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
