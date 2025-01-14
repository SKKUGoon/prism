create schema binance;

-- XRPUSDT
create table binance.features_xrpusdt_future (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_xrpusdt_future', 'time');
select add_retention_policy('binance.features_xrpusdt_future', INTERVAL '3 days');

create table binance.features_xrpusdt_spot (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_xrpusdt_spot', 'time');
select add_retention_policy('binance.features_xrpusdt_spot', INTERVAL '3 days');

-- BTCUSDT
create table binance.features_btcusdt_future (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_btcusdt_future', 'time');
select add_retention_policy('binance.features_btcusdt_future', INTERVAL '3 days');

create table binance.features_btcusdt_spot (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_btcusdt_spot', 'time');
select add_retention_policy('binance.features_btcusdt_spot', INTERVAL '3 days');

-- BNBUSDT
create table binance.features_bnbusdt_future (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_bnbusdt_future', 'time');
select add_retention_policy('binance.features_bnbusdt_future', INTERVAL '3 days');

create table binance.features_bnbusdt_spot (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_bnbusdt_spot', 'time');
select add_retention_policy('binance.features_bnbusdt_spot', INTERVAL '3 days');

-- DOGEUSDT
create table binance.features_dogeusdt_future (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_dogeusdt_future', 'time');
select add_retention_policy('binance.features_dogeusdt_future', INTERVAL '3 days');

create table binance.features_dogeusdt_spot (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_dogeusdt_spot', 'time');
select add_retention_policy('binance.features_dogeusdt_spot', INTERVAL '3 days');

-- SUIUSDT
create table binance.features_suiusdt_future (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_suiusdt_future', 'time');
select add_retention_policy('binance.features_suiusdt_future', INTERVAL '3 days');

create table binance.features_suiusdt_spot (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    obi FLOAT4 not null,
    obi_001 FLOAT4 not null,
    obi_002 FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null,
    tib_id text not null,
    tib_ts TIMESTAMP not null,
    tib_te TIMESTAMP not null,
    tib_ps FLOAT4 not null,
    tib_pe FLOAT4 not null,
    tib_imb FLOAT4 not null
);

select create_hypertable('binance.features_suiusdt_spot', 'time');
select add_retention_policy('binance.features_suiusdt_spot', INTERVAL '3 days');
