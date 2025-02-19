create schema binance;

-- XRPUSDT
create table binance.features_xrpusdt_future3 (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_005p FLOAT4 not null,
    ob_spread FLOAT4 not null,
    
    tib_id_hist text not null,
    tib_imb_hist FLOAT4 not null,
    tib_thres_hist FLOAT4 not null,
    tib_aggr_hist FLOAT4 not null,
    tib_aggr_v_hist FLOAT4 not null,
    tib_imb_curr FLOAT4 not null,
    tib_thres_curr FLOAT4 not null,
    tib_vwap_curr FLOAT4 not null,
    
    vmb_id_hist text not null,
    vmb_imb_hist FLOAT4 not null,
    vmb_thres_hist FLOAT4 not null,
    vmb_aggr_hist FLOAT4 not null,
    vmb_aggr_v_hist FLOAT4 not null,
    vmb_imb_curr FLOAT4 not null,
    vmb_thres_curr FLOAT4 not null,
    vmb_vwap_curr FLOAT4 not null,
    vmb_cvd_curr FLOAT4 not null,
    dib_id_hist text not null,
    dib_imb_hist FLOAT4 not null,
    dib_thres_hist FLOAT4 not null,
    dib_aggr_hist FLOAT4 not null,
    dib_aggr_v_hist FLOAT4 not null,
    dib_imb_curr FLOAT4 not null,
    dib_thres_curr FLOAT4 not null,
    dib_vwap_curr FLOAT4 not null,
    dib_cvd_curr FLOAT4 not null
);

select create_hypertable('binance.features_xrpusdt_future3', 'time');
select add_retention_policy('binance.features_xrpusdt_future3', INTERVAL '3 days');

create table binance.features_xrpusdt_spot3 (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    ob_spread FLOAT4 not null,  
    obi_005p FLOAT4 not null,
    
    tib_id_hist text not null,
    tib_imb_hist FLOAT4 not null,
    tib_thres_hist FLOAT4 not null,
    tib_aggr_hist FLOAT4 not null,
    tib_aggr_v_hist FLOAT4 not null,
    tib_imb_curr FLOAT4 not null,
    tib_thres_curr FLOAT4 not null,
    tib_vwap_curr FLOAT4 not null,
    
    vmb_id_hist text not null,
    vmb_imb_hist FLOAT4 not null,
    vmb_thres_hist FLOAT4 not null,
    vmb_aggr_hist FLOAT4 not null,
    vmb_aggr_v_hist FLOAT4 not null,
    vmb_imb_curr FLOAT4 not null,
    vmb_thres_curr FLOAT4 not null,
    vmb_vwap_curr FLOAT4 not null,
    vmb_cvd_curr FLOAT4 not null,
    
    dib_id_hist text not null,
    dib_imb_hist FLOAT4 not null,
    dib_thres_hist FLOAT4 not null,
    dib_aggr_hist FLOAT4 not null,
    dib_aggr_v_hist FLOAT4 not null,
    dib_imb_curr FLOAT4 not null,
    dib_thres_curr FLOAT4 not null,
    dib_vwap_curr FLOAT4 not null,
    dib_cvd_curr FLOAT4 not null
);

select create_hypertable('binance.features_xrpusdt_spot3', 'time');
select add_retention_policy('binance.features_xrpusdt_spot3', INTERVAL '3 days');


-- Strategy 1
create table crypto.strategy1_layer (
    time TIMESTAMP not null,
    price FLOAT4,
    best_bid_price FLOAT4,
    best_bid_volume FLOAT4,
    best_ask_price FLOAT4,
    best_ask_volume FLOAT4,
    best_bid_ask_spread FLOAT4,  
    total_bid_volume FLOAT4,
    total_ask_volume FLOAT4,
    market_buy_size FLOAT4,
    market_sell_size FLOAT4
);

select create_hypertable('crypto.strategy1_layer', 'time');
select add_retention_policy('crypto.strategy1_layer', INTERVAL '5 days');
