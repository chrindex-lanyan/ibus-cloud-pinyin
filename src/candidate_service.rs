use std::sync::Arc;

use tokio::sync::Mutex;

use super::{candidate::Candidate, ibus_proxy::IBusProxy, ibus_variants::IBusLookupTable};

struct State {
    candidates: Vec<Candidate>,
    page: usize,
}

impl State {
    pub fn new() -> Self {
        State {
            candidates: Vec::new(),
            page: 0,
        }
    }
}

unsafe impl Sync for State {} // State is safe to share between threads

pub struct CandidateService {
    lt_size: usize,
    state: Mutex<State>,
    ibus: Arc<Mutex<IBusProxy>>,
}

impl CandidateService {
    pub fn new(ibus: Arc<Mutex<IBusProxy>>) -> CandidateService {
        CandidateService {
            lt_size: 5,
            state: Mutex::new(State::new()),
            ibus,
        }
    }

    pub async fn in_session(&self) -> bool {
        self.state.lock().await.candidates.len() != 0
    }

    pub async fn set_candidates(&self, candidates: &[Candidate]) {
        let mut state = self.state.lock().await;

        state.candidates.clear();
        for candidate in candidates {
            state.candidates.push(candidate.clone());
        }

        // IBus
        let page = state.page;
        let start = 0 + self.lt_size * page; // inclusive
        let end = start + self.lt_size; // exclusive

        let to_show = if state.candidates.is_empty() {
            // 没有候选词，发送空表
            IBusLookupTable::from_nothing()
        } else 
        {
            // 计算实际的结束索引，防止越界
            let actual_end = std::cmp::min(end, state.candidates.len());
            // 只要 start < actual_end，就表示当前页有候选词
            if start < actual_end {
                IBusLookupTable::from_candidates(&state.candidates[start..actual_end])
            } else {
                // 理论上 state.page=0 时不会到这里，除非 lt_size=0
                IBusLookupTable::from_nothing()
            }
        };

        //println!("Candidates to show in table: {:?}", to_show); 
        
        drop(state);

        self.ibus
            .lock()
            .await
            .update_lookup_table(to_show, true)
            .await;
    }

    pub async fn page_into(&self) -> (bool, Option<usize>) {
        let mut state = self.state.lock().await;

        let potential_start = (state.page + 1) * self.lt_size;
        if potential_start >= state.candidates.len() {
             // 需要更多候选词
            drop(state); // 释放锁
            // 计算至少需要多少个才能填满下一页
            let min_needed = potential_start + 1; // 至少需要有下一页的第一个
            return (false, Some(min_needed)); // (IsEnough, HowManyAtLeastDoWeNeed)
        }

        // 确认可以翻页
        state.page += 1;
        let start = state.page * self.lt_size;
        let end = start + self.lt_size;
        let actual_end = std::cmp::min(end, state.candidates.len()); // 边界检查

        let to_show = if start < actual_end {
            IBusLookupTable::from_candidates(&state.candidates[start..actual_end])
        } else {
            // 理论上不应发生，因为上面检查过 potential_start
            IBusLookupTable::from_nothing()
        };

        drop(state);

        self.ibus
            .lock()
            .await
            .update_lookup_table(to_show, true)
            .await;
        return (true, None);
    }

    pub async fn page_back(&self) {
        let mut state = self.state.lock().await;

        if state.page == 0 {
            drop(state); // 释放锁
            return; // 已经是第一页
        }
        state.page -= 1;
        let start = state.page * self.lt_size;
        let end = start + self.lt_size;
        // 对于回翻，end 不会越界，因为之前的页肯定存在
        let actual_end = std::cmp::min(end, state.candidates.len()); // 仍然做检查以防万一

        let to_show = if start < actual_end {
            IBusLookupTable::from_candidates(&state.candidates[start..actual_end])
        } else {
            IBusLookupTable::from_nothing()
        };

        drop(state);

        self.ibus
            .lock()
            .await
            .update_lookup_table(to_show, true)
            .await;
    }

    pub async fn select(&self, ith: usize) {
        let state = self.state.lock().await;
        let idx = ith - 1 + state.page * self.lt_size;
        let text = state.candidates[idx].word.clone();

        drop(state);

        self.ibus.lock().await.commit_text(&text).await;

        self.clear().await;
    }

    pub async fn clear(&self) {
        let mut state = self.state.lock().await;
        state.candidates.clear();
        state.page = 0;

        drop(state);

        self.ibus
            .lock()
            .await
            .update_lookup_table(IBusLookupTable::from_nothing(), false)
            .await;
    }
}
