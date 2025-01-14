use crate::prism::bar_tick_imbalance::Tib;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Bar {
    pub tib_queue: VecDeque<Tib>,
    tib_max_capa: usize,
}

#[allow(dead_code)]
impl Bar {
    pub fn new(tick_imbalance_max_capacity: usize) -> Self {
        // When no Tib is generated
        Self {
            tib_queue: VecDeque::new(),
            tib_max_capa: tick_imbalance_max_capacity,
        }
    }

    pub fn update_bar(&mut self, bar: &Tib) {
        // Create a constant size queue
        if self.tib_queue.len() >= self.tib_max_capa {
            self.tib_queue.pop_front();
        }

        if let Some(last_bar) = self.tib_queue.back() {
            if last_bar.id == bar.id {
                return;
            }
        }
        self.tib_queue.push_back(bar.clone());
    }
}
