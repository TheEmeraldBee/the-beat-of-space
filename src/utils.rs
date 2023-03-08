use macroquad::file::FileError;
use macroquad::input::KeyCode;
use macroquad::math::Vec2;
use macroquad::prelude::{FilterMode, Rect};
use macroquad::prelude::Texture2D;
use macroquad::texture::load_texture;
use macroquad::time::get_frame_time;
use serde::{Deserialize, Serialize};

pub async fn quick_load_texture(path: &str) -> Result<Texture2D, FileError> {
    let texture = load_texture(path).await;

    let texture = match texture {
        Ok(tex) => tex,
        Err(error) => { return Err(error); }
    };
    texture.set_filter(FilterMode::Nearest);
    Ok(texture)
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Timer<T> {
    pub running: bool,
    pub timer: f32,
    max_timer: f32,
    pub data: T,
}

impl<T> Timer<T> where T: Send {
    pub fn new(time: f32, data: T) -> Self {
        Self {
            running: false,
            timer: time,
            max_timer: time,
            data,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn update(&mut self) {
        if self.running && self.timer >= 0.0 {
            self.timer -= get_frame_time()
        }
    }

    pub fn is_done(&self) -> bool {
        self.timer <= 0.0
    }

    pub fn percent_done(&self) -> f32 {
        1.0 - (self.timer / self.max_timer)
    }
}


#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Config {
    pub volume: f64,
    pub fullscreen: bool,
    pub controls: Controls,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Controls {
    pub up_arrow: u32,
    pub down_arrow: u32,
    pub left_arrow: u32,
    pub right_arrow: u32,
    pub ship_up: u32,
    pub ship_down: u32
}

pub fn u32_to_key_code(code: u32) -> KeyCode {
    match code {
        0 => KeyCode::Space,
        1 => KeyCode::Apostrophe,
        2 => KeyCode::Comma,
        3 => KeyCode::Minus,
        4 => KeyCode::Period,
        5 => KeyCode::Slash,
        6 => KeyCode::Key0,
        7 => KeyCode::Key1,
        8 => KeyCode::Key2,
        9 => KeyCode::Key3,
        10 => KeyCode::Key4,
        11 => KeyCode::Key5,
        12 => KeyCode::Key6,
        13 => KeyCode::Key7,
        14 => KeyCode::Key8,
        15 => KeyCode::Key9,
        16 => KeyCode::Semicolon,
        17 => KeyCode::Equal,
        18 => KeyCode::A,
        19 => KeyCode::B,
        20 => KeyCode::C,
        21 => KeyCode::D,
        22 => KeyCode::E,
        23 => KeyCode::F,
        24 => KeyCode::G,
        25 => KeyCode::H,
        26 => KeyCode::I,
        27 => KeyCode::J,
        28 => KeyCode::K,
        29 => KeyCode::L,
        30 => KeyCode::M,
        31 => KeyCode::N,
        32 => KeyCode::O,
        33 => KeyCode::P,
        34 => KeyCode::Q,
        35 => KeyCode::R,
        36 => KeyCode::S,
        37 => KeyCode::T,
        38 => KeyCode::U,
        39 => KeyCode::V,
        40 => KeyCode::W,
        41 => KeyCode::X,
        42 => KeyCode::Y,
        43 => KeyCode::Z,
        44 => KeyCode::LeftBracket,
        45 => KeyCode::Backslash,
        46 => KeyCode::RightBracket,
        47 => KeyCode::GraveAccent,
        48 => KeyCode::World1,
        49 => KeyCode::World2,
        50 => KeyCode::Escape,
        51 => KeyCode::Enter,
        52 => KeyCode::Tab,
        53 => KeyCode::Backspace,
        54 => KeyCode::Insert,
        55 => KeyCode::Delete,
        56 => KeyCode::Right,
        57 => KeyCode::Left,
        58 => KeyCode::Down,
        59 => KeyCode::Up,
        60 => KeyCode::PageUp,
        61 => KeyCode::PageDown,
        62 => KeyCode::Home,
        63 => KeyCode::End,
        64 => KeyCode::CapsLock,
        65 => KeyCode::ScrollLock,
        66 => KeyCode::NumLock,
        67 => KeyCode::PrintScreen,
        68 => KeyCode::Pause,
        69 => KeyCode::F1,
        70 => KeyCode::F2,
        71 => KeyCode::F3,
        72 => KeyCode::F4,
        73 => KeyCode::F5,
        74 => KeyCode::F6,
        75 => KeyCode::F7,
        76 => KeyCode::F8,
        77 => KeyCode::F9,
        78 => KeyCode::F10,
        79 => KeyCode::F11,
        80 => KeyCode::F12,
        81 => KeyCode::F13,
        82 => KeyCode::F14,
        83 => KeyCode::F15,
        84 => KeyCode::F16,
        85 => KeyCode::F17,
        86 => KeyCode::F18,
        87 => KeyCode::F19,
        88 => KeyCode::F20,
        89 => KeyCode::F21,
        90 => KeyCode::F22,
        91 => KeyCode::F23,
        92 => KeyCode::F24,
        93 => KeyCode::F25,
        94 => KeyCode::Kp0,
        95 => KeyCode::Kp1,
        96 => KeyCode::Kp2,
        97 => KeyCode::Kp3,
        98 => KeyCode::Kp4,
        99 => KeyCode::Kp5,
        100 => KeyCode::Kp6,
        101 => KeyCode::Kp7,
        102 => KeyCode::Kp8,
        103 => KeyCode::Kp9,
        104 => KeyCode::KpDecimal,
        105 => KeyCode::KpDivide,
        106 => KeyCode::KpMultiply,
        107 => KeyCode::KpSubtract,
        108 => KeyCode::KpAdd,
        109 => KeyCode::KpEnter,
        110 => KeyCode::KpEqual,
        111 => KeyCode::LeftShift,
        112 => KeyCode::LeftControl,
        113 => KeyCode::LeftAlt,
        114 => KeyCode::LeftSuper,
        115 => KeyCode::RightShift,
        116 => KeyCode::RightControl,
        117 => KeyCode::RightAlt,
        118 => KeyCode::RightSuper,
        119 => KeyCode::Menu,
        _ => panic!("Impossible! You Must've changed the number through the json!")
    }
}

pub fn key_code_to_u32(code: KeyCode) -> u32 {
    match code {
        KeyCode::Space => 0,
        KeyCode::Apostrophe => 1,
        KeyCode::Comma => 2,
        KeyCode::Minus => 3,
        KeyCode::Period => 4,
        KeyCode::Slash => 5,
        KeyCode::Key0 => 6,
        KeyCode::Key1 => 7,
        KeyCode::Key2 => 8,
        KeyCode::Key3 => 9,
        KeyCode::Key4 => 10,
        KeyCode::Key5 => 11,
        KeyCode::Key6 => 12,
        KeyCode::Key7 => 13,
        KeyCode::Key8 => 14,
        KeyCode::Key9 => 15,
        KeyCode::Semicolon => 16,
        KeyCode::Equal => 17,
        KeyCode::A => 18,
        KeyCode::B => 19,
        KeyCode::C => 20,
        KeyCode::D => 21,
        KeyCode::E => 22,
        KeyCode::F => 23,
        KeyCode::G => 24,
        KeyCode::H => 25,
        KeyCode::I => 26,
        KeyCode::J => 27,
        KeyCode::K => 28,
        KeyCode::L => 29,
        KeyCode::M => 30,
        KeyCode::N => 31,
        KeyCode::O => 32,
        KeyCode::P => 33,
        KeyCode::Q => 34,
        KeyCode::R => 35,
        KeyCode::S => 36,
        KeyCode::T => 37,
        KeyCode::U => 38,
        KeyCode::V => 39,
        KeyCode::W => 40,
        KeyCode::X => 41,
        KeyCode::Y => 42,
        KeyCode::Z => 43,
        KeyCode::LeftBracket => 44,
        KeyCode::Backslash => 45,
        KeyCode::RightBracket => 46,
        KeyCode::GraveAccent => 47,
        KeyCode::World1 => 48,
        KeyCode::World2 => 49,
        KeyCode::Escape => 50,
        KeyCode::Enter => 51,
        KeyCode::Tab => 52,
        KeyCode::Backspace => 53,
        KeyCode::Insert => 54,
        KeyCode::Delete => 55,
        KeyCode::Right => 56,
        KeyCode::Left => 57,
        KeyCode::Down => 58,
        KeyCode::Up => 59,
        KeyCode::PageUp => 60,
        KeyCode::PageDown => 61,
        KeyCode::Home => 62,
        KeyCode::End => 63,
        KeyCode::CapsLock => 64,
        KeyCode::ScrollLock => 65,
        KeyCode::NumLock => 66,
        KeyCode::PrintScreen => 67,
        KeyCode::Pause => 68,
        KeyCode::F1 => 69,
        KeyCode::F2 => 70,
        KeyCode::F3 => 71,
        KeyCode::F4 => 72,
        KeyCode::F5 => 73,
        KeyCode::F6 => 74,
        KeyCode::F7 => 75,
        KeyCode::F8 => 76,
        KeyCode::F9 => 77,
        KeyCode::F10 => 78,
        KeyCode::F11 => 79,
        KeyCode::F12 => 80,
        KeyCode::F13 => 81,
        KeyCode::F14 => 82,
        KeyCode::F15 => 83,
        KeyCode::F16 => 84,
        KeyCode::F17 => 85,
        KeyCode::F18 => 86,
        KeyCode::F19 => 87,
        KeyCode::F20 => 88,
        KeyCode::F21 => 89,
        KeyCode::F22 => 90,
        KeyCode::F23 => 91,
        KeyCode::F24 => 92,
        KeyCode::F25 => 93,
        KeyCode::Kp0 => 94,
        KeyCode::Kp1 => 95,
        KeyCode::Kp2 => 96,
        KeyCode::Kp3 => 97,
        KeyCode::Kp4 => 98,
        KeyCode::Kp5 => 99,
        KeyCode::Kp6 => 100,
        KeyCode::Kp7 => 101,
        KeyCode::Kp8 => 102,
        KeyCode::Kp9 => 103,
        KeyCode::KpDecimal => 104,
        KeyCode::KpDivide => 105,
        KeyCode::KpMultiply => 106,
        KeyCode::KpSubtract => 107,
        KeyCode::KpAdd => 108,
        KeyCode::KpEnter => 109,
        KeyCode::KpEqual => 110,
        KeyCode::LeftShift => 111,
        KeyCode::LeftControl => 112,
        KeyCode::LeftAlt => 113,
        KeyCode::LeftSuper => 114,
        KeyCode::RightShift => 115,
        KeyCode::RightControl => 116,
        KeyCode::RightAlt => 117,
        KeyCode::RightSuper => 118,
        KeyCode::Menu => 119,
        _ => 120
    }
}

pub fn is_hovering_rect(rect: Rect, mouse_pos: Vec2) -> bool {
    mouse_pos.x < rect.x + rect.w && mouse_pos.x > rect.x &&
        mouse_pos.y < rect.y + rect.h && mouse_pos.y > rect.y
}