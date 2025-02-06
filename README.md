# Prism High Frequency Trading Engine

## Setup

```bash
cargo build
```

## Dockerized

```bash
docker compose build
docker compose up
```


## Flow

![Flow](Prism_Engine_Flow_Design.png)


## Question

- Q1. Why one table setup? `ddl.sql` only has one table for future one table for spot.
- A1. Just tried to simulate how the `executor` and its `trade_computer` will see the data.




