use crate::prism::bar_tick_imbalance::TickImbalanceBar;
use crate::prism::bar_volume_imbalance::{VolumeImbalanceBar, VolumeType};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Bar {
    pub tick_imbalance_bar_queue: VecDeque<TickImbalanceBar>,
    tick_imbalance_bar_max_capa: usize,

    pub volume_imbalance_bar_both_queue: VecDeque<VolumeImbalanceBar>,
    volume_imbalance_bar_both_max_capa: usize,

    pub volume_imbalance_bar_maker_queue: VecDeque<VolumeImbalanceBar>,
    volume_imbalance_bar_maker_max_capa: usize,

    pub volume_imbalance_bar_taker_queue: VecDeque<VolumeImbalanceBar>,
    volume_imbalance_bar_taker_max_capa: usize,
}

#[allow(dead_code)]
impl Bar {
    pub fn new(max_capacity: usize) -> Self {
        // When no Tib is generated
        Self {
            tick_imbalance_bar_queue: VecDeque::new(),
            tick_imbalance_bar_max_capa: max_capacity,

            volume_imbalance_bar_both_queue: VecDeque::new(),
            volume_imbalance_bar_both_max_capa: max_capacity,

            volume_imbalance_bar_maker_queue: VecDeque::new(),
            volume_imbalance_bar_maker_max_capa: max_capacity,

            volume_imbalance_bar_taker_queue: VecDeque::new(),
            volume_imbalance_bar_taker_max_capa: max_capacity,
        }
    }

    pub fn update_tick_imbalance_bar(&mut self, bar: &TickImbalanceBar) {
        if self.tick_imbalance_bar_queue.len() >= self.tick_imbalance_bar_max_capa {
            self.tick_imbalance_bar_queue.pop_front();
        }

        if let Some(last_bar) = self.tick_imbalance_bar_queue.back() {
            if last_bar.id == bar.id {
                return;
            }
        }
        self.tick_imbalance_bar_queue.push_back(bar.clone());
    }

    pub fn update_volume_imbalance_bar(&mut self, bar: &VolumeImbalanceBar) {
        match bar.volume_type {
            VolumeType::Both => {
                if self.volume_imbalance_bar_both_queue.len()
                    >= self.volume_imbalance_bar_both_max_capa
                {
                    self.volume_imbalance_bar_both_queue.pop_front();
                }

                if let Some(last_bar) = self.volume_imbalance_bar_both_queue.back() {
                    if last_bar.id == bar.id {
                        return;
                    }
                }
                self.volume_imbalance_bar_both_queue.push_back(bar.clone());
            }
            VolumeType::Maker => {
                if self.volume_imbalance_bar_maker_queue.len()
                    >= self.volume_imbalance_bar_maker_max_capa
                {
                    self.volume_imbalance_bar_maker_queue.pop_front();
                }

                if let Some(last_bar) = self.volume_imbalance_bar_maker_queue.back() {
                    if last_bar.id == bar.id {
                        return;
                    }
                }
                self.volume_imbalance_bar_maker_queue.push_back(bar.clone());
            }
            VolumeType::Taker => {
                if self.volume_imbalance_bar_taker_queue.len()
                    >= self.volume_imbalance_bar_taker_max_capa
                {
                    self.volume_imbalance_bar_taker_queue.pop_front();
                }

                if let Some(last_bar) = self.volume_imbalance_bar_taker_queue.back() {
                    if last_bar.id == bar.id {
                        return;
                    }
                }
                self.volume_imbalance_bar_taker_queue.push_back(bar.clone());
            }
        }
    }
}
