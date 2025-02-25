pub mod binance;
pub mod upbit;

use std::collections::HashMap;

pub struct OrderbookUpdateStream {
    pub bids: HashMap<String, String>, // key: price, value: order id
    pub asks: HashMap<String, String>, // key: price, value: order id

    pub trade_time: u64,
    pub event_time: u64,
    pub last_update_exchange: String,
}

// #[allow(dead_code)]
// impl Orderbook {
//     // Receive `OrderbookUpdateStream`
//     // Process and send `OrderbookData`
//     pub fn new(rx: Receiver<OrderbookUpdateStream>, tx: Sender<OrderbookData>) -> Self {
//         Self {
//             bids: HashMap::new(),
//             asks: HashMap::new(),
//             trade_time: 0,
//             event_time: 0,
//             last_update_exchange: String::new(),
//             rx,
//             tx,
//         }
//     }

//     pub async fn listen(&mut self) {
//         while let Some(update) = self.rx.recv().await {
//             let (bids_diff, asks_diff) = self.calculate_diff(update);

//             // Send updated orderbook to prism
//             let data = OrderbookData {
//                 best_bid: (String::new(), String::new()),
//                 best_ask: (String::new(), String::new()),
//                 bids: self.bids.clone(),
//                 asks: self.asks.clone(),
//                 bids_diff,
//                 asks_diff,
//                 trade_time: self.trade_time,
//                 event_time: self.event_time,
//             };

//             if data.trade_time == 0 || data.event_time == 0 {
//                 continue;
//             }

//             if let Err(e) = self.tx.send(data).await {
//                 log::error!("Error sending orderbook data to prism: {}", e);
//             }
//         }
//     }

//     /// Updates the orderbook with new bid and ask data and calculates volume differences
//     ///
//     /// # Arguments
//     /// * `update` - New orderbook data containing bids, asks and timestamps
//     ///
//     /// # Returns
//     /// A tuple of HashMaps containing the volume differences for bids and asks:
//     /// * First HashMap contains bid price -> volume difference
//     /// * Second HashMap contains ask price -> volume difference
//     ///
//     /// # Details
//     /// For each bid and ask:
//     /// 1. If quantity is "0", removes the price level
//     /// 2. Otherwise updates the price level with new quantity
//     /// 3. Calculates volume difference between old and new quantity
//     /// 4. Stores the difference in the respective diff HashMap
//     ///
//     /// Also updates internal timestamps and exchange info
//     pub fn calculate_diff(
//         &mut self,
//         update: OrderbookUpdateStream,
//     ) -> (HashMap<String, String>, HashMap<String, String>) {
//         let mut bids_diff: HashMap<String, String> = HashMap::new();
//         let mut asks_diff: HashMap<String, String> = HashMap::new();

//         for (price, quantity) in update.bids.iter() {
//             let old_qty = self
//                 .bids
//                 .get(price)
//                 .unwrap_or(&"0.0".to_string())
//                 .parse::<f32>()
//                 .unwrap_or(0.0);
//             let new_qty = quantity.parse::<f32>().unwrap_or(0.0);
//             let diff = new_qty - old_qty;

//             bids_diff.insert(price.clone(), diff.to_string());

//             if quantity == "0" {
//                 self.bids.remove(price);
//             } else {
//                 self.bids.insert(price.clone(), quantity.clone());
//             }
//         }

//         for (price, quantity) in update.asks.iter() {
//             let old_qty = self
//                 .asks
//                 .get(price)
//                 .unwrap_or(&"0.0".to_string())
//                 .parse::<f32>()
//                 .unwrap_or(0.0);
//             let new_qty = quantity.parse::<f32>().unwrap_or(0.0);
//             let diff = new_qty - old_qty;
//             asks_diff.insert(price.clone(), diff.to_string());

//             if quantity == "0" {
//                 self.asks.remove(price);
//             } else {
//                 self.asks.insert(price.clone(), quantity.clone());
//             }
//         }

//         self.trade_time = update.trade_time;
//         self.event_time = update.event_time;
//         self.last_update_exchange = update.last_update_exchange.to_string();

//         (bids_diff, asks_diff)
//     }
// }

// #[allow(dead_code)]
// impl OrderbookData {
//     pub fn update_best_bid_ask(&mut self) {
//         let best_bid = self
//             .bids
//             .iter()
//             .filter(|(_, bq)| bq.parse::<f32>().unwrap_or(0f32) > 0f32)
//             .max_by(|a, b| {
//                 a.0.parse::<f32>()
//                     .unwrap_or(0f32)
//                     .partial_cmp(&b.0.parse::<f32>().unwrap_or(0f32))
//                     .unwrap()
//             });

//         let best_ask = self
//             .asks
//             .iter()
//             .filter(|(_, aq)| aq.parse::<f32>().unwrap_or(0f32) > 0f32)
//             .min_by(|a, b| {
//                 a.0.parse::<f32>()
//                     .unwrap_or(f32::MAX)
//                     .partial_cmp(&b.0.parse::<f32>().unwrap_or(f32::MAX))
//                     .unwrap()
//             });

//         self.best_bid = (best_bid.unwrap().0.clone(), best_bid.unwrap().1.clone());
//         self.best_ask = (best_ask.unwrap().0.clone(), best_ask.unwrap().1.clone());
//     }

//     pub fn near_price_bid_ask_activity(
//         &mut self,
//         price: f32,
//         margin: f32,
//     ) -> (HashMap<String, String>, HashMap<String, String>) {
//         let mut bid_activity: HashMap<String, String> = HashMap::new();
//         let mut ask_activity: HashMap<String, String> = HashMap::new();

//         // Calculate bid activity within margin
//         for (price_str, quantity) in self.bids.iter() {
//             let bid_price = price_str.parse::<f32>().unwrap_or(0.0);
//             if bid_price < price && bid_price > price * (1f32 - margin) {
//                 bid_activity.insert(price_str.clone(), quantity.clone());
//             }
//         }

//         // Calculate ask activity within margin
//         for (price_str, quantity) in self.asks.iter() {
//             let ask_price = price_str.parse::<f32>().unwrap_or(0.0);
//             if ask_price > price && ask_price < price * (1f32 + margin) {
//                 ask_activity.insert(price_str.clone(), quantity.clone());
//             }
//         }

//         (bid_activity, ask_activity)
//     }
// }
