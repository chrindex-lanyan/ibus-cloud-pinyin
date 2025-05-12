use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{keys::Key, mode_switcher::KeyContent};

use super::ibus_proxy::IBusProxy;

pub struct NumberService {
    ibus: Arc<Mutex<IBusProxy>>,
}

impl NumberService {
    pub fn new(ibus: Arc<Mutex<IBusProxy>>) -> NumberService {
        NumberService { ibus }
    }

    pub async fn handle_number(&self, key_content : KeyContent) {
        let n = key_content.key
            .to_usize()
            .expect("This key cannot be converted to a usize.");

        let text = n.to_string();

        self.ibus.lock().await.commit_text(&text).await;
    }
}
