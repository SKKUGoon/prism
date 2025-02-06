use crate::prism::bar_dollar_imbalance::{DollarImbalanceBar, DollarVolumeType};
use crate::prism::bar_tick_imbalance::TickImbalanceBar;
use crate::prism::bar_volume_imbalance::{VolumeImbalanceBar, VolumeType};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Bar {
    // Manages all the possible bars
    // 1. Tick Imbalance Bar
    // - (Explain)
    // 2. Volume Imbalance Bar
    // - (Explain)
    // 3. Dollar Imbalance Bar
    // - (Explain)
    pub tick_imbalance_bar_queue: VecDeque<TickImbalanceBar>,
    tick_imbalance_bar_max_capa: usize,

    pub volume_imbalance_bar_both_queue: VecDeque<VolumeImbalanceBar>,
    volume_imbalance_bar_both_max_capa: usize,

    pub volume_imbalance_bar_maker_queue: VecDeque<VolumeImbalanceBar>,
    volume_imbalance_bar_maker_max_capa: usize,

    pub volume_imbalance_bar_taker_queue: VecDeque<VolumeImbalanceBar>,
    volume_imbalance_bar_taker_max_capa: usize,

    pub dollar_imbalance_bar_both_queue: VecDeque<DollarImbalanceBar>,
    dollar_imbalance_bar_both_max_capa: usize,

    pub dollar_imbalance_bar_maker_queue: VecDeque<DollarImbalanceBar>,
    dollar_imbalance_bar_maker_max_capa: usize,

    pub dollar_imbalance_bar_taker_queue: VecDeque<DollarImbalanceBar>,
    dollar_imbalance_bar_taker_max_capa: usize,
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

            dollar_imbalance_bar_both_queue: VecDeque::new(),
            dollar_imbalance_bar_both_max_capa: max_capacity,

            dollar_imbalance_bar_maker_queue: VecDeque::new(),
            dollar_imbalance_bar_maker_max_capa: max_capacity,

            dollar_imbalance_bar_taker_queue: VecDeque::new(),
            dollar_imbalance_bar_taker_max_capa: max_capacity,
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
        log::info!(
            "Tick imbalance bar updated: VWAP vs Price: {:?} vs {:?}",
            bar.vwap,
            bar.pc, // Always use the closing price
        );
    }

    pub fn update_volume_imbalance_bar(&mut self, bar: &VolumeImbalanceBar) {
        let (queue, max_capacity) = match bar.volume_type {
            VolumeType::Both => (
                &mut self.volume_imbalance_bar_both_queue,
                self.volume_imbalance_bar_both_max_capa,
            ),
            VolumeType::Maker => (
                &mut self.volume_imbalance_bar_maker_queue,
                self.volume_imbalance_bar_maker_max_capa,
            ),
            VolumeType::Taker => (
                &mut self.volume_imbalance_bar_taker_queue,
                self.volume_imbalance_bar_taker_max_capa,
            ),
        };

        // Pop front if queue is at capacity
        if queue.len() >= max_capacity {
            queue.pop_front();
        }

        // Skip if bar already exists
        if let Some(last_bar) = queue.back() {
            if last_bar.id == bar.id {
                return;
            }
        }

        queue.push_back(bar.clone());
    }

    pub fn update_dollar_imbalance_bar(&mut self, bar: &DollarImbalanceBar) {
        let (queue, max_capacity) = match bar.volume_type {
            DollarVolumeType::Both => (
                &mut self.dollar_imbalance_bar_both_queue,
                self.dollar_imbalance_bar_both_max_capa,
            ),
            DollarVolumeType::Maker => (
                &mut self.dollar_imbalance_bar_maker_queue,
                self.dollar_imbalance_bar_maker_max_capa,
            ),
            DollarVolumeType::Taker => (
                &mut self.dollar_imbalance_bar_taker_queue,
                self.dollar_imbalance_bar_taker_max_capa,
            ),
        };

        // Pop front if queue is at capacity
        if queue.len() >= max_capacity {
            queue.pop_front();
        }

        // Skip if bar already exists
        if let Some(last_bar) = queue.back() {
            if last_bar.id == bar.id {
                return;
            }
        }

        queue.push_back(bar.clone());
    }
}
