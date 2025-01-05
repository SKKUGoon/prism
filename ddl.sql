create table binance.features (
    time TIMESTAMP not null,
    source text not null,
    price FLOAT4 not null,
    maker_quantity FLOAT4 not null,
    taker_quantity FLOAT4 not null,
    aggressive FLOAT4 not null,
    obi_005 FLOAT4 not null,
    obi_010 FLOAT4 not null
);

select create_hypertable('binance.features', 'time');