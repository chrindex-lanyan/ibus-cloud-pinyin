use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{keys::Key, mode_switcher::KeyContent};

use super::ibus_proxy::IBusProxy;

pub struct SymbolService {
    pub(crate) ibus: Arc<Mutex<IBusProxy>>,
}

impl SymbolService {
    pub fn new(ibus: Arc<Mutex<IBusProxy>>) -> SymbolService {
        SymbolService { ibus }
    }

    pub async fn handle_symbol(&self, key_content:KeyContent) {

        if key_content.flags.is_release {
            return 
        }

        let fw_puctuation = key_content.key
            .to_full_width_string()
            .expect("This key cannot be converted to fullwidth string.");

        self.ibus.lock().await.commit_text(&fw_puctuation).await;
    }
}
