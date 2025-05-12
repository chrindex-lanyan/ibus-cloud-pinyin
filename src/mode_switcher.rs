use std::{
    fmt::{self},
    sync::{Arc, Mutex},
};

use crate::keys::Key;
pub struct ModeSwitcher {
    mode: Arc<Mutex<Mode>>,
    last: Arc<Mutex<Key>>,
}

impl ModeSwitcher {
    pub fn new() -> ModeSwitcher {
        ModeSwitcher {
            mode: Arc::new(Mutex::new(Mode::Pinyin)),
            last: Arc::new(Mutex::new(Key::a)),
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
        _keycode: u32,
        state: u32,
    ) -> ModeSwitcherReturn {

        let key = match Key::from_u32(keyval) {
            Some(key) => key,
            None => {
                return ModeSwitcherReturn::Done(false);
            },
        };
        let last = self.last();
        
        // State flags
        let flags = self.decode_flag(state);
        let is_modifier = flags.is_ctrl
            || flags.is_alt
            || flags.is_super
            || flags.is_hyper
            || flags.is_meta
            || flags.is_lock;

        if !flags.is_release {
            self.set_last(key);
        }

        println!("last={:?}, current={:?}, is_modifier={}", &last, &key, is_modifier);

        if is_modifier || !flags.is_release {
            // User control like ctrl+v that has nothing to do with us.
            return ModeSwitcherReturn::Done(false);
        }

        if (key == Key::Shift) 
            && (flags.is_release) 
            && (last == Key::Shift) 
        {
            match self.mode() {
                Mode::English => {
                    self.set_mode(Mode::Pinyin);
                    println!("EN->PY");
                },
                Mode::Pinyin => {
                    self.set_mode(Mode::English);
                    println!("PY->EN");
                },
            }
            return ModeSwitcherReturn::Continue(key, true);
        }

        match self.mode() {
            Mode::English => ModeSwitcherReturn::Done(true),
            Mode::Pinyin => ModeSwitcherReturn::Continue(key, false),
        }
    }

    fn mode(&self) -> Mode {
        *self.mode.lock().expect("Failed to lock mode.")
    }

    fn set_mode(&self, val: Mode) {
        let mut mode = self.mode.lock().expect("Failed to lock mode.");
        *mode = val;
    }

    fn last(&self) -> Key {
        *self.last.lock().expect("Failed to lock last.")
    }

    fn set_last(&self, val: Key) {
        let mut last = self.last.lock().expect("Failed to lock last.");
        *last = val;
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

#[derive(Clone, Copy)]
pub enum ModeSwitcherReturn {
    Continue(Key, bool),
    Done(bool),
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    English,
    Pinyin,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Flags {
    is_shift: bool,
    is_lock: bool,
    is_ctrl: bool,
    is_alt: bool,
    is_mod2: bool,
    is_mod3: bool,
    is_mod4: bool,
    is_mod5: bool,
    is_btn1: bool,
    is_btn2: bool,
    is_btn3: bool,
    is_btn4: bool,
    is_btn5: bool,
    is_handled: bool,
    is_ignored: bool,
    is_super: bool,
    is_hyper: bool,
    is_meta: bool,
    is_release: bool,
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
