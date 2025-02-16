# Prism High Frequency Trading Engine Documentation

## Overview
Prism is a high-frequency trading engine written in Rust that processes real-time market data from multiple cryptocurrency exchanges (Binance and Upbit). It handles both spot and futures markets data streams, processes them, and can execute trades or store data based on configuration.

## Architecture

### Data Flow
1. **Data Ingestion**
   - Websocket connections receive market data from exchanges
   - Data flows: Websocket → Data → Engine → Executor (or database) → Trade order

2. **Channel Structure**
   Channel naming convention: `tx(rx)_(fut/spt)_(ob/agg)_data`
   - `fut`: Futures market
   - `spt`: Spot market
   - `ob`: Orderbook data
   - `agg`: Aggregated trade data

3. **Core Components**

#### Stream Managers
- `FutureStream`: Handles futures market data
- `SpotStream`: Handles spot market data
- Supports multiple exchange-specific implementations:
  - `BinanceStreams`
  - `UpbitStreams`

#### Data Processing
- `PrismTradeManager`: Core trading logic implementation
- Supports both real-time trade execution and data dumping

#### Database Integration
- Uses TimescaleDB for data storage
- Batch writing capability for efficient data persistence

### Key Features

1. **Multi-Exchange Support**
   - Binance (Futures and Spot)
   - Upbit (Spot - KRW, BTC, USDT pairs)

2. **Data Types**
   - Orderbook data
   - Aggregated trades
   - Mark price (Futures)
   - Liquidation data (Futures)

3. **Configuration**
   - Environment-based configuration
   - Configurable channel capacities
   - Optional data dumping mode

4. **Graceful Shutdown**
   - Handles Ctrl+C signal
   - Proper task cleanup
   - Organized shutdown sequence

## Setup and Deployment

### Local Development
```bash
cargo build
```

### Docker Deployment
```bash
docker compose build
docker compose up
```

## Technical Details

### Channel System
The system uses a sophisticated channel system for inter-thread communication:
- Separate channels for futures and spot markets
- Dedicated channels for different data types (orderbook, trades)
- System channels for execution and database operations

### Threading Model
- Uses Tokio for async runtime
- Implements task-based concurrency
- Each major component runs in its own task

### Error Handling
- Comprehensive logging system
- Error propagation through channels
- Graceful error recovery mechanisms

## Database Structure
- Single table design for futures
- Single table design for spot markets
- Optimized for time-series data using TimescaleDB

## Best Practices
1. Always use the provided channel capacity from environment configuration
2. Monitor system resources when running with data dumping enabled
3. Implement proper error handling for websocket disconnections
4. Regular monitoring of task health through logging

## Limitations and Considerations
1. Memory usage with high channel capacities
2. Network bandwidth requirements for multiple websocket connections
3. Database write performance under heavy load
4. Processing latency considerations for HFT operations

## Future Improvements
1. Additional exchange support
2. Enhanced error recovery mechanisms
3. Performance optimization for critical paths
4. Advanced trading strategies implementation
5. Monitoring and alerting system integration

## Flow

![Flow](Prism_Engine_Flow_Design.png)


## Question

- Q1. Why one table setup? `ddl.sql` only has one table for future one table for spot.
- A1. Just tried to simulate how the `executor` and its `trade_computer` will see the data.




