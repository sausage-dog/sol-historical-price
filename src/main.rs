use std::{error::Error};

pub mod prep;
pub use prep::*;

fn main() -> Result<(), Box<dyn Error>> {

    let source = ["./solana_data_1s/spot/monthly/klines/SOLUSDT/1s","./solana_data_1s/spot/daily/klines/SOLUSDT/1s"];
    let dest = "./solana_historical_price.dat";
    
    parse_binance(dest, &source).or_else(| err| {
        eprintln!("Error in merging: {}", err);
        Err(err)
    })?;   

    sample_readouts()?;

    Ok(())
}