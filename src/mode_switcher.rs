use std::{
    fmt::{self},
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::keys::Key;

#[derive(Clone)]
pub struct KeyContent{
    pub key : Key,
    pub flags : Flags,
    pub key_code : u32,
}

pub struct ModeSwitcher {
    mode: Arc<Mutex<Mode>>,
    last: Arc<Mutex<KeyContent>>,
}

impl ModeSwitcher {
    pub fn new() -> ModeSwitcher {
        ModeSwitcher {
            mode: Arc::new(Mutex::new(Mode::English)),
            last: Arc::new(Mutex::new(KeyContent{
                key : Key::a,
                flags : Flags {
                    is_ignored : true,
                    ..Default::default()
                },
                key_code: 0,
            })),
        }
    }

    pub async fn process_key_event (&self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> ModeSwitcherReturn {

        self.process_key_event_new(keyval, keycode, state).await
    }

    pub async fn process_key_event_new(
        &self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> ModeSwitcherReturn {

        let key = match Key::from_u32(keyval) {
            Some(key) => key,
            None => {
                return ModeSwitcherReturn::Done(false);
            },
        };
        let last = self.last().await;
        let flags = self.decode_flag(state);
        let key_content = KeyContent { 
            key: key, 
            flags: flags.clone(), 
            key_code: keycode 
        };
        self.set_last(key_content.clone()).await;
        
        // State flags
        let is_modifier = flags.is_ctrl
            || flags.is_alt
            || flags.is_super
            || flags.is_hyper
            || flags.is_meta
            || flags.is_lock;
        
        if is_modifier && self.mode().await == Mode::English{
            return ModeSwitcherReturn::Done(false);
        }

        if (key_content.key == Key::Shift) && (key_content.flags.is_release) 
            && (last.key == Key::Shift) && (!last.flags.is_release)
        {
            match self.mode().await {
                Mode::English => {
                    self.set_mode(Mode::Pinyin).await;
                },
                Mode::Pinyin => {
                    self.set_mode(Mode::English).await;
                },
            }
            return ModeSwitcherReturn::SwitchMode;
        }

        match self.mode().await {
            Mode::English => ModeSwitcherReturn::Done(false),
            Mode::Pinyin => {
                ModeSwitcherReturn::Continue(key_content)
            },
        }
    }

    async fn mode(&self) -> Mode {
        let x = self.mode.lock().await;
        x.clone()
    }

    async fn set_mode(&self, val: Mode) {
        let mut mode = self.mode.lock().await;
        *mode = val;
    }

    async fn last(&self) -> KeyContent {
        let l = self.last.lock().await;
        l.clone()
    }

    async fn set_last(&self, val: KeyContent) {
        let mut l = self.last.lock().await;
        *l = val;
    }

    fn get_kth_bit(&self, n: u32, k: u32) -> bool {
        (n & (1 << k)) >> k == 1
    }

    fn decode_flag(&self, flag: u32) -> Flags {
        Flags {
            is_shift: self.get_kth_bit(flag, 0),
            is_lock: self.get_kth_bit(flag, 1),
            is_ctrl: self.get_kth_bit(flag, 2),
            is_alt: self.get_kth_bit(flag, 3),
            is_mod2: self.get_kth_bit(flag, 4),
            is_mod3: self.get_kth_bit(flag, 5),
            is_mod4: self.get_kth_bit(flag, 6),
            is_mod5: self.get_kth_bit(flag, 7),
            is_btn1: self.get_kth_bit(flag, 8),
            is_btn2: self.get_kth_bit(flag, 9),
            is_btn3: self.get_kth_bit(flag, 10),
            is_btn4: self.get_kth_bit(flag, 11),
            is_btn5: self.get_kth_bit(flag, 12),
            is_handled: self.get_kth_bit(flag, 24),
            is_ignored: self.get_kth_bit(flag, 25),
            is_super: self.get_kth_bit(flag, 26),
            is_hyper: self.get_kth_bit(flag, 27),
            is_meta: self.get_kth_bit(flag, 28),
            is_release: self.get_kth_bit(flag, 30),
        }
    }
}

#[derive(Clone)]
pub enum ModeSwitcherReturn {
    Continue(KeyContent),
    Done(bool),
    SwitchMode,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    English,
    Pinyin,
}

#[derive(Debug, Default,Clone)]
pub struct Flags {
    pub is_shift: bool,
    pub is_lock: bool,
    pub is_ctrl: bool,
    pub is_alt: bool,
    pub is_mod2: bool,
    pub is_mod3: bool,
    pub is_mod4: bool,
    pub is_mod5: bool,
    pub is_btn1: bool,
    pub is_btn2: bool,
    pub is_btn3: bool,
    pub is_btn4: bool,
    pub is_btn5: bool,
    pub is_handled: bool,
    pub is_ignored: bool,
    pub is_super: bool,
    pub is_hyper: bool,
    pub is_meta: bool,
    pub is_release: bool,
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Shift:{} Ctrl:{} Alt:{} Release:{}]",
            self.is_shift, self.is_ctrl, self.is_alt, self.is_release
        )
    }
}
