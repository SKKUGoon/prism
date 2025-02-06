create schema binance;

-- XRPUSDT
create table binance.features_xrpusdt_future3 (
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
    tib_imb_curr FLOAT4 not null,
    tib_thres_curr FLOAT4 not null,
    tib_vwap_curr FLOAT4 not null,
    
    vmb_id_hist text not null,
    vmb_imb_hist FLOAT4 not null,
    vmb_thres_hist FLOAT4 not null,
    vmb_imb_curr FLOAT4 not null,
    vmb_thres_curr FLOAT4 not null,
    vmb_vwap_curr FLOAT4 not null,
    
    vmm_id_hist text not null,
    vmm_imb_hist FLOAT4 not null,
    vmm_thres_hist FLOAT4 not null,
    vmm_imb_curr FLOAT4 not null,
    vmm_thres_curr FLOAT4 not null,
    vmm_vwap_curr FLOAT4 not null,
    
    vmt_id_hist text not null,
    vmt_imb_hist FLOAT4 not null,
    vmt_thres_hist FLOAT4 not null,
    vmt_imb_curr FLOAT4 not null,
    vmt_thres_curr FLOAT4 not null,
    vmt_vwap_curr FLOAT4 not null,
    
    dib_id_hist text not null,
    dib_imb_hist FLOAT4 not null,
    dib_thres_hist FLOAT4 not null,
    dib_imb_curr FLOAT4 not null,
    dib_thres_curr FLOAT4 not null,
    dib_vwap_curr FLOAT4 not null
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
    tib_imb_curr FLOAT4 not null,
    tib_thres_curr FLOAT4 not null,
    tib_vwap_curr FLOAT4 not null,
    
    vmb_id_hist text not null,
    vmb_imb_hist FLOAT4 not null,
    vmb_thres_hist FLOAT4 not null,
    vmb_imb_curr FLOAT4 not null,
    vmb_thres_curr FLOAT4 not null,
    vmb_vwap_curr FLOAT4 not null,
    
    vmm_id_hist text not null,
    vmm_imb_hist FLOAT4 not null,
    vmm_thres_hist FLOAT4 not null,
    vmm_imb_curr FLOAT4 not null,
    vmm_thres_curr FLOAT4 not null,
    vmm_vwap_curr FLOAT4 not null,
    
    vmt_id_hist text not null,
    vmt_imb_hist FLOAT4 not null,
    vmt_thres_hist FLOAT4 not null,
    vmt_imb_curr FLOAT4 not null,
    vmt_thres_curr FLOAT4 not null,
    vmt_vwap_curr FLOAT4 not null,
    
    dib_id_hist text not null,
    dib_imb_hist FLOAT4 not null,
    dib_thres_hist FLOAT4 not null,
    dib_imb_curr FLOAT4 not null,
    dib_thres_curr FLOAT4 not null,
    dib_vwap_curr FLOAT4 not null
);

select create_hypertable('binance.features_xrpusdt_spot3', 'time');
select add_retention_policy('binance.features_xrpusdt_spot3', INTERVAL '3 days');

-- BTCUSDT
-- create table binance.features_btcusdt_future (
--     time TIMESTAMP not null,
--     source text not null,
--     price FLOAT4 not null,
--     maker_quantity FLOAT4 not null,
--     taker_quantity FLOAT4 not null,
--     obi FLOAT4 not null,
--     obi_005p FLOAT4 not null,
--     obi_01p FLOAT4 not null,
--     obi_02p FLOAT4 not null,
--     obi_05p FLOAT4 not null,
--     tib_id text not null,
--     tib_imb FLOAT4 not null,
--     vmb_id text not null,
--     vmb_imb FLOAT4 not null,
--     vmm_id text not null,
--     vmm_imb FLOAT4 not null,
--     vmt_id text not null,
--     vmt_imb FLOAT4 not null
-- );

-- select create_hypertable('binance.features_btcusdt_future', 'time');
-- select add_retention_policy('binance.features_btcusdt_future', INTERVAL '3 days');

-- create table binance.features_btcusdt_spot (
--     time TIMESTAMP not null,
--     source text not null,
--     price FLOAT4 not null,
--     maker_quantity FLOAT4 not null,
--     taker_quantity FLOAT4 not null,
--     obi FLOAT4 not null,
--     obi_005p FLOAT4 not null,
--     obi_01p FLOAT4 not null,
--     obi_02p FLOAT4 not null,
--     obi_05p FLOAT4 not null,
--     tib_id text not null,
--     tib_imb FLOAT4 not null,
--     vmb_id text not null,
--     vmb_imb FLOAT4 not null,
--     vmm_id text not null,
--     vmm_imb FLOAT4 not null,
--     vmt_id text not null,
--     vmt_imb FLOAT4 not null
-- );

-- select create_hypertable('binance.features_btcusdt_spot', 'time');
-- select add_retention_policy('binance.features_btcusdt_spot', INTERVAL '3 days');

-- -- BNBUSDT
-- create table binance.features_bnbusdt_future (
--     time TIMESTAMP not null,
--     source text not null,
--     price FLOAT4 not null,
--     maker_quantity FLOAT4 not null,
--     taker_quantity FLOAT4 not null,
--     obi FLOAT4 not null,
--     obi_005p FLOAT4 not null,
--     obi_01p FLOAT4 not null,
--     obi_02p FLOAT4 not null,
--     obi_05p FLOAT4 not null,
--     tib_id text not null,
--     tib_imb FLOAT4 not null,
--     vmb_id text not null,
--     vmb_imb FLOAT4 not null,
--     vmm_id text not null,
--     vmm_imb FLOAT4 not null,
--     vmt_id text not null,
--     vmt_imb FLOAT4 not null
-- );

-- select create_hypertable('binance.features_bnbusdt_future', 'time');
-- select add_retention_policy('binance.features_bnbusdt_future', INTERVAL '3 days');

-- create table binance.features_bnbusdt_spot (
--     time TIMESTAMP not null,
--     source text not null,
--     price FLOAT4 not null,
--     maker_quantity FLOAT4 not null,
--     taker_quantity FLOAT4 not null,
--     obi FLOAT4 not null,
--     obi_005p FLOAT4 not null,
--     obi_01p FLOAT4 not null,
--     obi_02p FLOAT4 not null,
--     obi_05p FLOAT4 not null,
--     tib_id text not null,
--     tib_imb FLOAT4 not null,
--     vmb_id text not null,
--     vmb_imb FLOAT4 not null,
--     vmm_id text not null,
--     vmm_imb FLOAT4 not null,
--     vmt_id text not null,
--     vmt_imb FLOAT4 not null
-- );

-- select create_hypertable('binance.features_bnbusdt_spot', 'time');
-- select add_retention_policy('binance.features_bnbusdt_spot', INTERVAL '3 days');

-- -- DOGEUSDT
-- create table binance.features_dogeusdt_future (
--     time TIMESTAMP not null,
--     source text not null,
--     price FLOAT4 not null,
--     maker_quantity FLOAT4 not null,
--     taker_quantity FLOAT4 not null,
--     obi FLOAT4 not null,
--     obi_005p FLOAT4 not null,
--     obi_01p FLOAT4 not null,
--     obi_02p FLOAT4 not null,
--     obi_05p FLOAT4 not null,
--     tib_id text not null,
--     tib_imb FLOAT4 not null,
--     vmb_id text not null,
--     vmb_imb FLOAT4 not null,
--     vmm_id text not null,
--     vmm_imb FLOAT4 not null,
--     vmt_id text not null,
--     vmt_imb FLOAT4 not null
-- );

-- select create_hypertable('binance.features_dogeusdt_future', 'time');
-- select add_retention_policy('binance.features_dogeusdt_future', INTERVAL '3 days');

-- create table binance.features_dogeusdt_spot (
--     time TIMESTAMP not null,
--     source text not null,
--     price FLOAT4 not null,
--     maker_quantity FLOAT4 not null,
--     taker_quantity FLOAT4 not null,
--     obi FLOAT4 not null,
--     obi_005p FLOAT4 not null,
--     obi_01p FLOAT4 not null,
--     obi_02p FLOAT4 not null,
--     obi_05p FLOAT4 not null,
--     tib_id text not null,
--     tib_imb FLOAT4 not null,
--     vmb_id text not null,
--     vmb_imb FLOAT4 not null,
--     vmm_id text not null,
--     vmm_imb FLOAT4 not null,
--     vmt_id text not null,
--     vmt_imb FLOAT4 not null
-- );

-- select create_hypertable('binance.features_dogeusdt_spot', 'time');
-- select add_retention_policy('binance.features_dogeusdt_spot', INTERVAL '3 days');

-- -- SUIUSDT
-- create table binance.features_suiusdt_future (
--     time TIMESTAMP not null,
--     source text not null,
--     price FLOAT4 not null,
--     maker_quantity FLOAT4 not null,
--     taker_quantity FLOAT4 not null,
--     obi FLOAT4 not null,
--     obi_005p FLOAT4 not null,
--     obi_01p FLOAT4 not null,
--     obi_02p FLOAT4 not null,
--     obi_05p FLOAT4 not null,
--     tib_id text not null,
--     tib_imb FLOAT4 not null,
--     vmb_id text not null,
--     vmb_imb FLOAT4 not null,
--     vmm_id text not null,
--     vmm_imb FLOAT4 not null,
--     vmt_id text not null,
--     vmt_imb FLOAT4 not null
-- );

-- select create_hypertable('binance.features_suiusdt_future', 'time');
-- select add_retention_policy('binance.features_suiusdt_future', INTERVAL '3 days');

-- create table binance.features_suiusdt_spot (
--     time TIMESTAMP not null,
--     source text not null,
--     price FLOAT4 not null,
--     maker_quantity FLOAT4 not null,
--     taker_quantity FLOAT4 not null,
--     obi FLOAT4 not null,
--     obi_005p FLOAT4 not null,
--     obi_01p FLOAT4 not null,
--     obi_02p FLOAT4 not null,
--     obi_05p FLOAT4 not null,
--     tib_id text not null,
--     tib_imb FLOAT4 not null,
--     vmb_id text not null,
--     vmb_imb FLOAT4 not null,
--     vmm_id text not null,
--     vmm_imb FLOAT4 not null,
--     vmt_id text not null,
--     vmt_imb FLOAT4 not null
-- );

-- select create_hypertable('binance.features_suiusdt_spot', 'time');
-- select add_retention_policy('binance.features_suiusdt_spot', INTERVAL '3 days');
