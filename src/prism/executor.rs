#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PrismaTradeConfig {
    leverage: u8,
    max_leverage: u8,

    loss_cut: f32,
    take_profit: f32,

    max_position_size: f32,
}

// trade_config: PrismaTradeConfig {
//     leverage: 1,
//     max_leverage: 30,
//     loss_cut: 0.5,
//     take_profit: 0.10,
//     max_position_size: 1000.0,
// },
