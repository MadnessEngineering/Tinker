use iced::{
    widget::{button, column, container, row, text, text_input},
    Application, Command, Element, Length, Settings, Theme,
};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    
    App::run(Settings::default())
}

#[derive(Debug, Default)]
struct App {
    input_value: String,
    output_value: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    Submit,
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Tinker")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::Submit => {
                self.output_value = format!("You entered: {}", self.input_value);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            row![
                text_input("Enter text...", &self.input_value)
                    .on_input(Message::InputChanged)
                    .padding(10)
                    .width(Length::Fill),
                button("Submit")
                    .on_press(Message::Submit)
                    .padding(10)
            ]
            .spacing(10)
            .padding(10),
            text(&self.output_value).size(20)
        ]
        .spacing(20)
        .padding(20)
        .max_width(800);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
