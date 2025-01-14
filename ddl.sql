create schema binance;

create table binance.features_future (
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
    tib_imb FLOAT4 not null,
)

select create_hypertable('binance.features_future', 'time');
SELECT add_retention_policy('binance.features_future', INTERVAL '3 days');

create table binance.features_spot (
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
    tib_imb FLOAT4 not null,
)

select create_hypertable('binance.features_spot', 'time');
SELECT add_retention_policy('binance.features_spot', INTERVAL '3 days');
