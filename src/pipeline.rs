use zbus::Connection;

use crate::{dispatcher::Dispatcher, mode_switcher::{ModeSwitcher, ModeSwitcherReturn}};

pub struct Pipeline {
    mode_switcher: ModeSwitcher,
    dispatcher: Dispatcher,
}

impl Pipeline {
    pub fn new(conn: &Connection) -> Pipeline {
        Pipeline {
            mode_switcher: ModeSwitcher::new(),
            dispatcher: Dispatcher::new(conn),
        }
    }

    pub async fn accept(&self, keyval: u32, keycode: u32, state: u32) -> bool {
        let output = self
            .mode_switcher
            .process_key_event(keyval, keycode, state)
            .await;

        match output {
            ModeSwitcherReturn::Continue(key, should_reset) => 
                self.dispatcher.on_input(key, should_reset).await,
            ModeSwitcherReturn::Done(has_handled) => 
                has_handled,
        }
    }
}
