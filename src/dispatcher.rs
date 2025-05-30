use std::{default, sync::Arc};

use zbus::Connection;

use super::ibus_proxy::IBusProxy;
use crate::{candidate::Candidate, keys::Key, mode_switcher::{Flags, KeyContent}, preedit_service::PreeditService};
use tokio::sync::Mutex;

use super::{
    candidate_service::CandidateService, cloud_pinyin_client::CloudPinyinClient,
    number_service::NumberService, symbol_service::SymbolService,
};

pub struct Dispatcher {
    pub candidate_svc: CandidateService,
    pub preedit_svc: PreeditService,
    symbol_svc: SymbolService,
    number_svc: NumberService,
    client: CloudPinyinClient,
    ibus: Arc<Mutex<IBusProxy>>,
    level: Vec<usize>,
}

impl Dispatcher {
    pub fn new(conn: &Connection) -> Dispatcher {
        let ibus: Arc<Mutex<IBusProxy>> = Arc::new(Mutex::new(IBusProxy::new(conn)));
        Dispatcher {
            candidate_svc: CandidateService::new(ibus.clone()),
            preedit_svc: PreeditService::new(ibus.clone()),
            symbol_svc: SymbolService::new(ibus.clone()),
            number_svc: NumberService::new(ibus.clone()),
            client: CloudPinyinClient::new(),
            ibus: ibus.clone(),
            level: vec![11, 21, 41, 81, 161, 321, 641, 1281],
        }
    }

    pub async fn on_input(&self, key_content: KeyContent) -> bool {
        
        match key_content.key {
            Key::a
            | Key::b
            | Key::c
            | Key::d
            | Key::e
            | Key::f
            | Key::g
            | Key::h
            | Key::i
            | Key::j
            | Key::k
            | Key::l
            | Key::m
            | Key::n
            | Key::o
            | Key::p
            | Key::q
            | Key::r
            | Key::s
            | Key::t
            | Key::u
            | Key::v
            | Key::w
            | Key::x
            | Key::y
            | Key::z => return self.handle_pinyin(key_content).await,

            Key::_0
            | Key::_1
            | Key::_2
            | Key::_3
            | Key::_4
            | Key::_5
            | Key::_6
            | Key::_7
            | Key::_8
            | Key::_9 => {
                if self.candidate_svc.in_session().await {
                    return self.handle_select(key_content).await;
                } else {
                    self.number_svc.handle_number(key_content).await;
                    return true;
                }
            }

            Key::Comma
            | Key::Period
            | Key::SemiColon
            | Key::Colon
            | Key::SingleQuote
            | Key::DoubleQuote
            | Key::BracketOpen
            | Key::BracketClose
            | Key::QuestionMark
            | Key::BackSlash
            | Key::ExclamationMark
            | Key::Ellipsis => {
                self.symbol_svc.handle_symbol(key_content).await;
                return true;
            }

            Key::Space
            | Key::Enter
            | Key::Minus
            | Key::Equal
            | Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::Backspace
            | Key::Escape => return self.handle_control(key_content).await,
            
            Key::Shift | Key::Ctrl | Key::Alt => panic!("Unexpected control keys received."),
            
            Key::A
            | Key::B
            | Key::C
            | Key::D
            | Key::E
            | Key::F
            | Key::G
            | Key::H
            | Key::I
            | Key::J
            | Key::K
            | Key::L
            | Key::M
            | Key::N
            | Key::O
            | Key::P
            | Key::Q
            | Key::R
            | Key::S
            | Key::T
            | Key::U
            | Key::V
            | Key::W
            | Key::X
            | Key::Y
            | Key::Z => panic!("We do not handle uppercase letters."),
        }
    }

    pub async fn handle_pinyin(&self, key_content: KeyContent) -> bool {
        
        if key_content.flags.is_release {
            return true;
        }

        let c = key_content.key.to_char().expect("A-Z cannot be converted to a char.");

        self.preedit_svc.push(c).await;
        
        //let preedit = self.preedit_svc.to_string().await;
        //let candidates = self.client.query_candidates(&preedit, self.level[0]).await;

        // 
        // DEBUG //////////////////
        // 
        let candidates = vec![
            Candidate {
                word: "你好".to_string(),
                annotation: "".to_string(), // 可以留空或添加调试信息
                matched_len: None,          // 对于固定列表，通常设为 None
            },
            Candidate {
                word: "世界".to_string(),
                annotation: "".to_string(),
                matched_len: None,
            },
        ];

        //println!("4. Fetched candidates: {:?}", candidates);
        self.candidate_svc.set_candidates(&candidates).await;

        true
    }

    pub async fn handle_select(&self, key_content: KeyContent) -> bool {
        
        if key_content.flags.is_release {
            return true
        }

        self.preedit_svc.clear().await;

        let i = key_content.key.to_usize().expect("Failed to conver the key to usize.");
        self.candidate_svc.select(i).await;
        self.candidate_svc.clear().await;

        true
    }

    pub async fn handle_control(&self, key_content: KeyContent) -> bool {
        if !self.candidate_svc.in_session().await {
            return false;
        }

        match key_content.key {
            Key::Space => return self.handle_select(KeyContent { 
                key: Key::_1, 
                flags: Flags {
                    is_ignored : true,
                    ..Default::default()
                }, 
                key_code: 0 ,
            }).await,

            Key::Enter => {
                let preedit = self.preedit_svc.to_string().await;
                self.preedit_svc.clear().await;
                self.candidate_svc.clear().await;
                self.ibus.lock().await.commit_text(&preedit).await;

                return true;
            }
            Key::Minus => {
                self.candidate_svc.page_back().await;

                return true;
            }
            Key::Equal => {
                let (enough, min_needed) = self.candidate_svc.page_into().await;
                if !enough {
                    let min = min_needed
                        .expect("Not enough to fill lookup table but min_needed is None.");

                    let mut to_load = 0;
                    for qty in &self.level {
                        if qty >= &min {
                            to_load = *qty;
                            break;
                        }
                    }

                    let candidates = self
                        .client
                        .query_candidates(&self.preedit_svc.to_string().await, to_load)
                        .await;
                    self.candidate_svc.set_candidates(&candidates).await;
                }

                return true;
            }
            Key::Up => return false,    // For now, ingore
            Key::Down => return false,  // For now, ignore
            Key::Left => return false,  // For now, ignore
            Key::Right => return false, // For now, ignore
            Key::Backspace => {
                let popped = self.preedit_svc.pop().await;

                if popped.is_none() {
                    return false;
                }

                let preedit: String = self.preedit_svc.to_string().await;

                let candidates = self.client.query_candidates(&preedit, self.level[0]).await;

                self.candidate_svc.set_candidates(&candidates).await;

                return true;
            }
            Key::Escape => {
                self.preedit_svc.clear().await;
                self.candidate_svc.clear().await;

                return true;
            }
            _ => panic!("Invalid control key."),
        }
    }
}
