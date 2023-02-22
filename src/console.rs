use macroquad::{
    prelude::{
        draw_rectangle, draw_text, screen_height, screen_width, vec2, Color, BLANK, GRAY, RED,
        WHITE, YELLOW,
    },
    texture::Image,
    ui::{hash, root_ui, widgets::InputText, Skin},
};
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;

// when we show the console, we delay it by a frame so that we don't capture the keystroke that opened it
enum ConsoleVisibility {
    Hidden,
    VisibleNextFrame,
    Visible,
}

pub enum ConsoleEntryType {
    Input,
    Output,
    Info,
    Warning,
}

pub static CONSOLE: Lazy<Mutex<Console>> = Lazy::new(|| Mutex::new(Console::new()));

pub struct Console {
    visibility: ConsoleVisibility,
    history: VecDeque<(ConsoleEntryType, String)>,
    current_input: String,
}

impl Console {
    fn new() -> Self {
        Self {
            visibility: ConsoleVisibility::Hidden,
            history: VecDeque::new(),
            current_input: "".to_owned(),
        }
    }

    pub fn is_visible(&self) -> bool {
        matches!(self.visibility, ConsoleVisibility::Visible)
    }

    pub fn toggle_visible(&mut self) {
        match self.visibility {
            ConsoleVisibility::Hidden => self.visibility = ConsoleVisibility::VisibleNextFrame,
            ConsoleVisibility::VisibleNextFrame => (),
            ConsoleVisibility::Visible => self.visibility = ConsoleVisibility::Hidden,
        }
    }

    pub fn force_visible(&mut self) {
        self.visibility = ConsoleVisibility::Visible;
    }

    pub fn escape(&mut self) {
        self.current_input = "".to_owned();
        self.visibility = ConsoleVisibility::Hidden;
    }

    pub fn execute(&mut self) {
        if self.current_input.is_empty() {
            return;
        }
        self.add(self.current_input.clone(), ConsoleEntryType::Input);
        self.add(
            format!("Executing console command: {}", self.current_input),
            ConsoleEntryType::Output,
        );
        self.current_input = "".to_owned();
    }

    pub fn add(&mut self, msg: String, typ: ConsoleEntryType) {
        self.history.push_front((typ, msg));
    }

    pub fn draw(&mut self) {
        if matches!(self.visibility, ConsoleVisibility::Visible) {
            let rows = (screen_height() * 0.4 / 16.0).ceil();
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                rows * 16.0 + 21.0,
                Color::new(0.0, 0.0, 0.0, 0.5),
            );
            while self.history.len() > rows as usize {
                self.history.pop_back();
            }
            for ii in 0..self.history.len() {
                let (typ, msg) = &self.history[ii];
                draw_text(
                    msg,
                    4.0,
                    (rows - ii as f32) * 16.0 as f32,
                    16.0,
                    match typ {
                        ConsoleEntryType::Input => WHITE,
                        ConsoleEntryType::Output => GRAY,
                        ConsoleEntryType::Info => YELLOW,
                        ConsoleEntryType::Warning => RED,
                    },
                );
            }
            let bg = Image::gen_image_color(1, 1, BLANK);
            let style = root_ui()
                .style_builder()
                .background(bg)
                .color(BLANK)
                .text_color(WHITE)
                .build();
            let skin = Skin {
                label_style: style.clone(),
                editbox_style: style.clone(),
                group_style: style,
                ..root_ui().default_skin()
            };
            root_ui().push_skin(&skin);

            let id_prompt = hash!();
            InputText::new(id_prompt)
                .position(vec2(0., rows as f32 * 16.0 + 1.0))
                .ui(&mut root_ui(), &mut self.current_input);
            root_ui().set_input_focus(id_prompt);
        }
        if matches!(self.visibility, ConsoleVisibility::VisibleNextFrame) {
            self.visibility = ConsoleVisibility::Visible;
        }
    }
}
