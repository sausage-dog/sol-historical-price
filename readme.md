# SOL price

Run `pip3 install` for any missing packages.
Likwise `cargo build` for rust libraries.

Python script fetches data from binance API, takes about 30mins.
Rust script puts it into a single file with u32 for timestamp (in seconds) and price (three decimal points precision as integer).

1. Run the python script `python3 binance_sol_price_fetch.py`
2. Run rust script `cargo run`

Final data is about 1.3GB

## Binance data


| Open Time (ms) | Open Price | High Price | Low Price | Close Price | Volume (SOL) | Close Time (ms) | Quote Volume (USDT) | Number of Trades | Taker Buy Base Volume | Taker Buy Quote Volume | Ignore |
|----------------|------------|------------|-----------|-------------|--------------|-----------------|--------------------|-----------------|-----------------------|------------------------|--------|
| 1597125600000  | 2.85000000 | 2.85000000 | 2.85000000| 2.85000000  | 3.60000000   | 1597125600999   | 10.26000000        | 1               | 3.60000000            | 10.26000000            | 0      |
| 1597125601000  | 2.85000000 | 2.85000000 | 2.85000000| 2.85000000  | 0.00000000   | 1597125601999   | 0.00000000         | 0               | 0.00000000            | 0.00000000             | 0      |
| 1597125602000  | 2.85000000 | 2.85000000 | 2.85000000| 2.85000000  | 0.00000000   | 1597125602999   | 0.00000000         | 0               | 0.00000000            | 0.00000000             | 0      |
| 1597125603000  | 2.85000000 | 2.85000000 | 2.85000000| 2.85000000  | 0.00000000   | 1597125603999   | 0.00000000         | 0               | 0.00000000            | 0.00000000             | 0      |
| 1597125604000  | 2.85000000 | 2.85000000 | 2.85000000| 2.85000000  | 0.00000000   | 1597125604999   | 0.00000000         | 0               | 0.00000000            | 0.00000000             | 0      |

### Column Descriptions

- **Open Time**: Unix timestamp in milliseconds (start of 1-second interval)
- **Open Price**: SOL price in USDT at interval start
- **High Price**: Highest SOL price during the 1-second interval
- **Low Price**: Lowest SOL price during the 1-second interval  
- **Close Price**: SOL price in USDT at interval end (most accurate current price)
- **Volume (SOL)**: Amount of SOL traded during this second
- **Close Time**: Unix timestamp in milliseconds (end of 1-second interval)
- **Quote Volume (USDT)**: Total USDT value traded during this second
- **Number of Trades**: Count of individual trades executed in this second
- **Taker Buy Base Volume**: SOL volume from market buy orders
- **Taker Buy Quote Volume**: USDT volume from market buy orders
- **Ignore**: Unused field (always 0)



### Note

Seconds is all over. Always same actual resolution, but varying degree of trailing zeros.
Same for price, which depends on magnitude.