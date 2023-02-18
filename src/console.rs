use macroquad::{
    prelude::{draw_rectangle, vec2, BLANK, BLUE, WHITE},
    texture::Image,
    ui::{hash, root_ui, widgets::Group, Skin},
};
use once_cell::sync::Lazy;
use std::sync::Mutex;

// when we show the console, we delay it by a frame so that we don't capture the keystroke that opened it
enum ConsoleVisibility {
    Hidden,
    VisibleNextFrame,
    Visible,
}

pub static CONSOLE: Lazy<Mutex<Console>> = Lazy::new(|| Mutex::new(Console::new()));

pub struct Console {
    visibility: ConsoleVisibility,
    current_input: String,
}

impl Console {
    fn new() -> Self {
        Self {
            visibility: ConsoleVisibility::Hidden,
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

    pub fn escape(&mut self) {
        self.current_input = "".to_owned();
        self.visibility = ConsoleVisibility::Hidden;
    }

    pub fn execute(&mut self) {
        println!("Executing console command: {}", self.current_input);
        self.current_input = "".to_owned();
    }

    pub fn draw(&mut self) {
        if matches!(self.visibility, ConsoleVisibility::Visible) {
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
            draw_rectangle(100., 100., 75., 75., BLUE);
            Group::new(hash!(), vec2(200.0, 100.0))
                .position(vec2(75.0, 75.0))
                .ui(&mut root_ui(), |ui| {
                    ui.input_text(id_prompt, "", &mut self.current_input)
                });
            root_ui().set_input_focus(id_prompt);
        }
        if matches!(self.visibility, ConsoleVisibility::VisibleNextFrame) {
            self.visibility = ConsoleVisibility::Visible;
        }
    }
}
