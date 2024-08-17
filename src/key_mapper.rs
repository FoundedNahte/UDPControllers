use anyhow::{Result, bail};
use device_query::{DeviceQuery, DeviceState};
use rkyv::{Archive, Serialize, Deserialize};
use std::sync::OnceLock;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml::de;
use device_query::keymap::Keycode;

fn key_hashmap() -> &'static HashMap<&'static str, Keycode> {
    static KEY_MAP: OnceLock<HashMap<&'static str, Keycode>> = OnceLock::new();
    KEY_MAP.get_or_init(|| {
        HashMap::from([
            ("Key0", Keycode::Key0),
            ("Key1", Keycode::Key1),
            ("Key2", Keycode::Key2),
            ("Key3", Keycode::Key3),
            ("Key4", Keycode::Key4),
            ("Key5", Keycode::Key5),
            ("Key6", Keycode::Key6),
            ("Key7", Keycode::Key7),
            ("Key8", Keycode::Key8),
            ("Key9", Keycode::Key9),
            ("A", Keycode::A),
            ("B", Keycode::B),
            ("C", Keycode::C),
            ("D", Keycode::D),
            ("E", Keycode::E),
            ("F", Keycode::F),
            ("G", Keycode::G),
            ("H", Keycode::H),
            ("I", Keycode::I),
            ("J", Keycode::J),
            ("K", Keycode::K),
            ("L", Keycode::L),
            ("M", Keycode::M),
            ("N", Keycode::N),
            ("O", Keycode::O),
            ("P", Keycode::P),
            ("Q", Keycode::Q),
            ("R", Keycode::R),
            ("S", Keycode::S),
            ("T", Keycode::T),
            ("U", Keycode::U),
            ("V", Keycode::V),
            ("W", Keycode::W),
            ("X", Keycode::X),
            ("Y", Keycode::Y),
            ("Z", Keycode::Z),
            ("F1", Keycode::F1),
            ("F2", Keycode::F2),
            ("F3", Keycode::F3),
            ("F4", Keycode::F4),
            ("F5", Keycode::F5),
            ("F6", Keycode::F6),
            ("F7", Keycode::F7),
            ("F8", Keycode::F8),
            ("F9", Keycode::F9),
            ("F10", Keycode::F10),
            ("F11", Keycode::F11),
            ("F12", Keycode::F12),
            ("F13", Keycode::F13),
            ("F14", Keycode::F14),
            ("F15", Keycode::F15),
            ("F16", Keycode::F16),
            ("F17", Keycode::F17),
            ("F18", Keycode::F18),
            ("F19", Keycode::F19),
            ("F20", Keycode::F20),
            ("Escape", Keycode::Escape),
            ("Space", Keycode::Space),
            ("LControl", Keycode::LControl),
            ("RControl", Keycode::RControl),
            ("LShift", Keycode::LShift),
            ("RShift", Keycode::RShift),
            ("LAlt", Keycode::LAlt),
            ("RAlt", Keycode::RAlt),
            ("Command", Keycode::Command),
            ("LOption", Keycode::LOption),
            ("ROption", Keycode::ROption),
            ("LMeta", Keycode::LMeta),
            ("RMeta", Keycode::RMeta),
            ("Enter", Keycode::Enter),
            ("Up", Keycode::Up),
            ("Down", Keycode::Down),
            ("Left", Keycode::Left),
            ("Right", Keycode::Right),
            ("Backspace", Keycode::Backspace),
            ("CapsLock", Keycode::CapsLock),
            ("Tab", Keycode::Tab),
            ("Home", Keycode::Home),
            ("End", Keycode::End),
            ("PageUp", Keycode::PageUp),
            ("PageDown", Keycode::PageDown),
            ("Insert", Keycode::Insert),
            ("Delete", Keycode::Delete),
            ("Numpad0", Keycode::Numpad0),
            ("Numpad1", Keycode::Numpad1),
            ("Numpad2", Keycode::Numpad2),
            ("Numpad3", Keycode::Numpad3),
            ("Numpad4", Keycode::Numpad4),
            ("Numpad5", Keycode::Numpad5),
            ("Numpad6", Keycode::Numpad6),
            ("Numpad7", Keycode::Numpad7),
            ("Numpad8", Keycode::Numpad8),
            ("Numpad9", Keycode::Numpad9),
            ("NumpadSubtract", Keycode::NumpadSubtract),
            ("NumpadAdd", Keycode::NumpadAdd),
            ("NumpadDivide", Keycode::NumpadDivide),
            ("NumpadMultiply", Keycode::NumpadMultiply),
            ("NumpadEquals", Keycode::NumpadEquals),
            ("NumpadEnter", Keycode::NumpadEnter),
            ("NumpadDecimal", Keycode::NumpadDecimal),
            ("Grave", Keycode::Grave),
            ("Minus", Keycode::Minus),
            ("Equal", Keycode::Equal),
            ("LeftBracket", Keycode::LeftBracket),
            ("RightBracket", Keycode::RightBracket),
            ("BackSlash", Keycode::BackSlash),
            ("Semicolon", Keycode::Semicolon),
            ("Apostrophe", Keycode::Apostrophe),
            ("Comma", Keycode::Comma),
            ("Dot", Keycode::Dot),
            ("Slash", Keycode::Slash)
        ])
    })
}

#[derive(Clone, Copy)]
enum ControllerAction {
    ThumbstickLX(i16),
    ThumbstickLY(i16),
    ThumbstickRX(i16),
    ThumbstickRY(i16),
    LTrigger(u8),
    RTrigger(u8),
    Button(u16),
}

fn controller_map() -> &'static HashMap<&'static str, ControllerAction> {
    static CONTROLLER_MAP: OnceLock<HashMap<&'static str, ControllerAction>> = OnceLock::new();
    CONTROLLER_MAP.get_or_init(|| {
        HashMap::from([
            ("LX+", ControllerAction::ThumbstickLX(29999)),
            ("LX-", ControllerAction::ThumbstickLX(-29999)),
            ("LY+", ControllerAction::ThumbstickLY(29999)),
            ("LY-", ControllerAction::ThumbstickLY(-29999)),
            ("RX+", ControllerAction::ThumbstickRX(29999)),
            ("RX-", ControllerAction::ThumbstickRX(-29999)),
            ("RY+", ControllerAction::ThumbstickRY(29999)),
            ("RY-", ControllerAction::ThumbstickRY(-29999)),
            ("UP", ControllerAction::Button(1)),
            ("DOWN", ControllerAction::Button(2)),
            ("LEFT", ControllerAction::Button(4)),
            ("RIGHT", ControllerAction::Button(8)),
            ("START", ControllerAction::Button(16)),
            ("BACK", ControllerAction::Button(32)),
            ("LTHUMB", ControllerAction::Button(64)),
            ("RTHUMB", ControllerAction::Button(128)),
            ("LB", ControllerAction::Button(256)),
            ("RB", ControllerAction::Button(512)),
            ("GUIDE", ControllerAction::Button(1024)),
            ("A", ControllerAction::Button(4096)),
            ("B", ControllerAction::Button(8192)),
            ("X", ControllerAction::Button(16384)),
            ("Y", ControllerAction::Button(32768)),
            ("LTRIGGER", 255),
            ("RTRIGGER", 255)
        ])
    })
}

#[derive(Archive, Deserialize, Serialize)]
pub(crate) enum ClientMessage {
    Hearbeat,
    Input(UserInput)
}

#[derive(Archive, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub(crate) struct UserInput {
    pub lx: i16,
    pub ly: i16,
    pub rx: i16,
    pub ry: i16,
    pub ltrigger: u8,
    pub rtrigger: u8,
    pub buttons: u16
}

pub(crate) struct KeyMapper {
    config: HashMap<Keycode, ControllerAction>,
    device_state: DeviceState
}

impl KeyMapper {
    pub fn new(config_path: &Path) -> Result<Self> {
        let config_string: String = fs::read_to_string(config_path)?;
        let config_stringmap: HashMap<String, String> = de::from_str(&config_string)?;

        let mut config: HashMap<Keycode, ControllerAction> = HashMap::new();
        for (key, action) in config_stringmap.into_iter() {
            match (key_hashmap().get(key.as_str()), controller_map().get(action.as_str())) {
                (Some(keycode), Some(controller_action)) => {
                    config.insert(*keycode, *controller_action);
                },
                (None, _) => {
                    bail!("Key not supported: {}", key);
                },
                (_, None) => {
                    bail!("Controller action not supported: {}", action) ;
                }
                _ => {}
            }
        }

        let device_state = DeviceState::new();

        Ok(KeyMapper {
            config,
            device_state
        })
    }

    pub fn get_input(&self) -> Result<UserInput> {
        let keys: Vec<Keycode> = self.device_state.get_keys();
        
        let mut lx: i16 = 0;
        let mut ly: i16 = 0;
        let mut rx: i16 = 0;
        let mut ry: i16 = 0;
        let mut ltrigger: u8 = 0;
        let mut rtrigger: u8 = 0;
        let mut buttons: u16 = 0;

        for keycode in keys {
            match self.config.get(&keycode) {
                Some(action) => {
                    match action {
                        ControllerAction::ThumbstickLX(direction) => lx = *direction,
                        ControllerAction::ThumbstickLY(direction) => ly = *direction,
                        ControllerAction::ThumbstickRX(direction) => rx = *direction,
                        ControllerAction::ThumbstickRY(direction) => ry = *direction,
                        ControllerAction::LTrigger(magnitude) => ltrigger = magnitude,
                        ControllerAction::RTrigger(magnitude) => rtrigger = magnitude,
                        ControllerAction::Button(button) => buttons |= button
                    }
                }
                None => {}
            }
        }
        
        Ok(UserInput {
            lx,
            ly,
            rx,
            ry,
            ltrigger,
            rtrigger,
            buttons
        })    
    }
}