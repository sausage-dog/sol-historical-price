use std::{    
    error::Error,
    ffi::OsString,
    fs::File,    
    io::{BufWriter, Write, BufReader, Read},
};
use std::fs;
use std::path::Path;
use std::io;

#[derive(Debug, Clone, Copy)]
pub struct SolanaPriceEntry {
    time: u32, // Timestamp in seconds since epoch.
    close_price: u32, // Integer price with 3 decimal places included.
}

fn filename(full_path: &str) -> String {
    if let Some(filename) = full_path.split('/').last() {
        filename.to_string()
    } else {
        full_path.to_string()
    }
}

fn fetch_files_alphabetically(dir_path: &str) -> io::Result<Vec<String>> {

    println!("Fetching files from directory: {}", dir_path);
    let path = Path::new(dir_path);    
    
    // Read the directory entries.
    let entries = fs::read_dir(path).or_else(|err| {
        eprintln!("Error reading directory {}: {}", dir_path, err);
        Err(err)
    })?;        
    
    // Collect entries into a vector first so we can count and process them.
    let entries: Vec<_> = entries.collect::<Result<Vec<_>, _>>()?;   
    
    // Collect all file paths (filter out directories).
    let mut files: Vec<String> = entries
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            
            // Only include files, not directories.
            if path.is_file() {                
                path.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    
    // Sort alphabetically by full path (case-insensitive).
    files.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    
    Ok(files)
}

/** Turns it one file. */
pub fn parse_binance(dest_file: &str, parse_files: &[&str]) -> Result<(), Box<dyn Error>> {    
 
    for dir in parse_files.iter() {
 
        // Start in the past, so using monthly data.
        let files = fetch_files_alphabetically(dir)?;        
 
        println!("Files found: {}", files.len());
 
        for file in files.iter() {                       
            let data = get_vector(&file).or_else(|err| {
                eprintln!("Error processing file {}: {}", file, err);
                Err(err)
            })?;            
 
            println!("Total entries in file {:?}: {}", filename(&file), data.len());
 
            // Append to the file.
            write_to_file(dest_file, &data)?;    
        }
    }     
 
    Ok(())
}

fn standardize_price(price: &str) -> Result<u32, Box<dyn Error>> {
    let dot_pos = price.find('.').unwrap_or(price.len());
    let before_dot = &price[..dot_pos];
    let after_dot = if dot_pos < price.len() {
        let remaining = &price[dot_pos + 1..];
        if remaining.len() >= 3 {
            &remaining[..3]
        } else {
            remaining
        }
    } else {
        ""
    };
    let combined = format!("{}{}", before_dot, after_dot);
    Ok(combined.parse::<u32>()?)
}

pub fn get_vector(path: &str) -> Result<Vec<SolanaPriceEntry>, Box<dyn Error>> {
    let mut vec: Vec<SolanaPriceEntry> = Vec::new();
    let file_path = OsString::from(path);
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);        
 
    for result in rdr.records() {
        let record = result?;
        if let Some(time) = record.get(0) {           
            if let Some(close_price) = record.get(4) {
 
                // Auto-detect format based on timestamp length.
                let drop = if time.len() == 13 { 3 } else { 6 };
 
                // Extract the seconds component.
                vec.push(SolanaPriceEntry{
                    time: time[..time.len() - drop].parse::<u32>().map_err(|e| {
                        eprintln!("TIMESTAMP PARSE FAILED: '{}'", &time[..time.len() - drop]);
                        e
                    })?,
                    close_price: standardize_price(close_price)?
                });          
            }
        }
    }      
 
    Ok(vec)
 }

pub fn write_to_file(path: &str, vec: &[SolanaPriceEntry]) -> Result<(), Box<dyn Error>> {
    let file_path = Path::new(path);

    // Check if file exists and validate timeline.
    if file_path.exists() {
        // Read first record from existing binary file to validate timeline.
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 8]; // 8 bytes for each u32
        
        if reader.read_exact(&mut buffer).is_ok() {
            let csv_timestamp = u32::from_le_bytes([
                buffer[0], buffer[1], buffer[2], buffer[3]
            ]);
            
            if let Some(first_entry) = vec.first() {
                let vec_timestamp = first_entry.time;
                if csv_timestamp > vec_timestamp {
                    // Convert to human readable (assuming seconds since epoch).
                    let csv_datetime = chrono::DateTime::from_timestamp(csv_timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| csv_timestamp.to_string());
                    let vec_datetime = chrono::DateTime::from_timestamp(vec_timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| vec_timestamp.to_string());
                        
                    return Err(format!(
                        "Timeline validation failed:\n  File starts at:   {} ({})\n  Vector starts at: {} ({})\nFile should start before or at the vector timestamp.",
                        csv_timestamp, csv_datetime,
                        vec_timestamp, vec_datetime
                    ).into());
                }
            }
        }
    }

    // Append binary data to the file.
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    
    let mut writer = BufWriter::new(file);

    for &pe in vec {
        // Write each u32 as 8 bytes in little-endian format
        writer.write_all(&pe.time.to_le_bytes())?;
        writer.write_all(&pe.close_price.to_le_bytes())?;
    }

    writer.flush()?;
    Ok(())
}

// Helper function to read binary data back (for testing/verification).
pub fn read_binary_file(path: &str) -> Result<Vec<(u32, u32)>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut vec = Vec::new();
    let mut buffer = [0u8; 8]; // 4 bytes for time (u32) + 4 bytes for price (u32).

    while reader.read_exact(&mut buffer).is_ok() {
        let time = u32::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3]
        ]);
        let price = u32::from_le_bytes([
            buffer[4], buffer[5], buffer[6], buffer[7],            
        ]);        
        vec.push((time, price));
    }

    Ok(vec)
}

pub fn sample_readouts() -> Result<(), Box<dyn Error>> {
    let readout = read_binary_file("/Users/wincentdulkowski/Files/0_Projects/rebalancing-data/solana_historical_price.dat").or_else(|err| {
        eprintln!("Error reading directory: {}", err);
        Err(err)
    })?;            
    
    let line_count = readout.len();

    // Print first 50.
    println!("\nFirst 50 entries:");
    for entry in readout.iter().take(50) {
        println!("{:?}", entry);
    }

    // Print middle 50.
    println!("\nMid 50 entries:");
    for entry in readout.iter().skip(line_count / 2).take(50) {
        println!("{:?}", entry);
    }
    // Print last 50.
    println!("\nLast 50 entries:");
    for entry in readout.iter().rev().take(50) {
        println!("{:?}", entry);
    }

    Ok(())
}
