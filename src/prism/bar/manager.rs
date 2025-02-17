use crate::prism::bar::{DollarImbalanceBar, TickImbalanceBar, VolumeImbalanceBar};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct BarManager {
    // Manages all the possible bars
    // 1. Tick Imbalance Bar
    // - (Explain)
    // 2. Volume Imbalance Bar
    // - (Explain)
    // 3. Dollar Imbalance Bar
    // - (Explain)
    pub tick_imbalance_bar_queue: VecDeque<TickImbalanceBar>,
    tick_imbalance_bar_max_capa: usize,

    pub volume_imbalance_bar_queue: VecDeque<VolumeImbalanceBar>,
    volume_imbalance_bar_max_capa: usize,

    pub dollar_imbalance_bar_queue: VecDeque<DollarImbalanceBar>,
    dollar_imbalance_bar_max_capa: usize,
}

#[allow(dead_code)]
impl BarManager {
    pub fn new(max_capacity: usize) -> Self {
        // When no Tib is generated
        Self {
            tick_imbalance_bar_queue: VecDeque::new(),
            tick_imbalance_bar_max_capa: max_capacity,

            volume_imbalance_bar_queue: VecDeque::new(),
            volume_imbalance_bar_max_capa: max_capacity,

            dollar_imbalance_bar_queue: VecDeque::new(),
            dollar_imbalance_bar_max_capa: max_capacity,
        }
    }

    pub fn update_tick_imbalance_bar(&mut self, bar: &TickImbalanceBar) {
        if self.tick_imbalance_bar_queue.len() >= self.tick_imbalance_bar_max_capa {
            self.tick_imbalance_bar_queue.pop_front();
        }

        if let Some(last_bar) = self.tick_imbalance_bar_queue.back() {
            if last_bar.bar.id == bar.bar.id {
                return;
            }
        }
        self.tick_imbalance_bar_queue.push_back(bar.clone());
    }

    pub fn update_volume_imbalance_bar(&mut self, bar: &VolumeImbalanceBar) {
        // Pop front if queue is at capacity
        if self.volume_imbalance_bar_queue.len() >= self.volume_imbalance_bar_max_capa {
            self.volume_imbalance_bar_queue.pop_front();
        }

        // Skip if bar already exists
        if let Some(last_bar) = self.volume_imbalance_bar_queue.back() {
            if last_bar.bar.id == bar.bar.id {
                return;
            }
        }

        self.volume_imbalance_bar_queue.push_back(bar.clone());
    }

    pub fn update_dollar_imbalance_bar(&mut self, bar: &DollarImbalanceBar) {
        // Pop front if queue is at capacity
        if self.dollar_imbalance_bar_queue.len() >= self.dollar_imbalance_bar_max_capa {
            self.dollar_imbalance_bar_queue.pop_front();
        }

        // Skip if bar already exists
        if let Some(last_bar) = self.dollar_imbalance_bar_queue.back() {
            if last_bar.bar.id == bar.bar.id {
                return;
            }
        }

        self.dollar_imbalance_bar_queue.push_back(bar.clone());
    }
}
