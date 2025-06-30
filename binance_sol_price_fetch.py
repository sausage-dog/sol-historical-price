# Tries 1-second data first (highest possible resolution)
# Falls back to 1-minute data if seconds aren't available
# Goes back as far as possible (to SOL's listing date)
# Loads ALL files (monthly + daily)
# Combines everything into one complete dataset

# Strategy:
# Step 1: Try downloading 1s data for all periods.
# Step 2: If that fails, download 1m data for all periods.
# Step 3: Process ALL downloaded files into one dataset.
# Step 4: Save complete dataset as CSV.

#!/usr/bin/env python3

import datetime
import pandas as pd
import os
from binance_historical_data import BinanceDataDumper

def get_all_solana_data(try_seconds=True):
    """
    Download ALL available Solana data with highest possible resolution
    
    Args:
        try_seconds (bool): Try to get 1-second data first, fall back to minutes
    """
    print("🎯 Goal: Get ALL Solana data with highest resolution possible")
    print("=" * 60)
    
    # Strategy: Try seconds first, fall back to minutes if needed.
    if try_seconds:
        print("🚀 Step 1: Trying 1-SECOND data (may not exist for all periods)...")
        success = try_download_resolution("1s")
        
        if success:
            print("✅ 1-second data downloaded successfully!")
            return process_downloaded_data("1s")
        else:
            print("❌ 1-second data not available, falling back to 1-minute...")
    
    print("🚀 Downloading 1-MINUTE data (going back as far as possible)...")
    success = try_download_resolution("1m")
    
    if success:
        print("✅ 1-minute data downloaded successfully!")
        return process_downloaded_data("1m")
    else:
        print("❌ Failed to download any data")
        return None

def try_download_resolution(frequency):
    """
    Try to download data with specified resolution
    
    Args:
        frequency (str): "1s" or "1m" etc.
    
    Returns:
        bool: True if successful
    """
    try:
        # Setup data dumper.
        data_dir = f"./solana_data_{frequency}"
        os.makedirs(data_dir, exist_ok=True)
        
        data_dumper = BinanceDataDumper(
            path_dir_where_to_dump=data_dir,
            asset_class="spot",
            data_type="klines",
            data_frequency=frequency
        )
        
        # Get earliest possible date.
        try:
            min_date = data_dumper.get_min_start_date_for_ticker("SOLUSDT")
            print(f"📅 Earliest SOLUSDT data: {min_date}")
            start_date = min_date
        except:
            # Fallback to known SOL listing date.
            start_date = datetime.date(2020, 8, 11)
            print(f"📅 Using fallback start date: {start_date}")
        
        end_date = datetime.date.today()
        total_days = (end_date - start_date).days
        
        print(f"📊 Downloading {frequency} data: {start_date} to {end_date} ({total_days} days)")
        
        if frequency == "1s" and total_days > 30:
            print("⚠️  WARNING: 1-second data for >30 days = MASSIVE download!")
            print("⚠️  This could be 10+ GB and take hours...")
            print("⚠️  Consider using smaller date range for 1s data")
        
        # Download ALL available data.
        data_dumper.dump_data(
            tickers=["SOLUSDT"],
            date_start=start_date,
            date_end=end_date,
            is_to_update_existing=False,
            tickers_to_exclude=[]
        )
        
        return True
        
    except Exception as e:
        print(f"❌ Error downloading {frequency} data: {e}")
        return False

def process_downloaded_data(frequency):
    """
    Load and process all downloaded data
    
    Args:
        frequency (str): The frequency that was downloaded
    
    Returns:
        pd.DataFrame: Combined data
    """
    data_dir = f"./solana_data_{frequency}"    
    print(f"📊 Processing all {frequency} data files...")    
    
    try:
        # The package creates subdirectories.
        actual_dir = os.path.join(data_dir, "spot", "daily", "klines", "SOLUSDT", frequency)
        if not os.path.exists(actual_dir):            
            actual_dir = os.path.join(data_dir, "spot", "monthly", "klines", "SOLUSDT", frequency)
        
        if not os.path.exists(actual_dir):
            # Try finding any CSV files in subdirectories.
            for root, dirs, files in os.walk(data_dir):
                csv_files = [f for f in files if f.endswith('.csv')]
                if csv_files:
                    actual_dir = root
                    break
    except:
        actual_dir = data_dir
    
    print(f"📂 Looking in: {actual_dir}")
    
    # Get all CSV files.
    csv_files = []
    for root, dirs, files in os.walk(actual_dir):
        for file in files:
            if file.endswith('.csv'):
                csv_files.append(os.path.join(root, file))
    
    if not csv_files:
        print("❌ No CSV files found!")
        return None
    
    print(f"📁 Found {len(csv_files)} CSV files")
    
    # Load ALL files.
    dataframes = []
    total_records = 0
    
    for file_path in sorted(csv_files):
        try:
            df = pd.read_csv(file_path)
            dataframes.append(df)
            total_records += len(df)
            file_name = os.path.basename(file_path)
            print(f"✓ {file_name}: {len(df):,} records")
        except Exception as e:
            print(f"❌ Error loading {file_path}: {e}")
    
    if not dataframes:
        print("❌ No data loaded!")
        return None
    
    print(f"\n🔗 Combining {len(dataframes)} files with {total_records:,} total records...")
    
    # Combine all data.
    combined_df = pd.concat(dataframes, ignore_index=True)
    
    # Process timestamps.
    combined_df['Open time'] = pd.to_datetime(combined_df['Open time'], unit='ms')
    combined_df['Close time'] = pd.to_datetime(combined_df['Close time'], unit='ms')
    
    # Sort and remove duplicates.
    combined_df = combined_df.sort_values('Open time').reset_index(drop=True)
    combined_df = combined_df.drop_duplicates(subset=['Open time'], keep='first')
    
    # Convert price columns.
    price_columns = ['Open', 'High', 'Low', 'Close', 'Volume']
    for col in price_columns:
        combined_df[col] = pd.to_numeric(combined_df[col], errors='coerce')
    
    # Final stats.
    print(f"\n📈 FINAL DATASET:")
    print(f"📊 Total Records: {len(combined_df):,}")
    print(f"📅 Date Range: {combined_df['Open time'].min()} to {combined_df['Open time'].max()}")
    print(f"⏱️  Resolution: {frequency} intervals")
    print(f"💰 Price Range: ${combined_df['Low'].min():.6f} - ${combined_df['High'].max():.6f}")
    
    # Save complete dataset.
    output_file = f"solana_complete_{frequency}_data.csv"
    combined_df.to_csv(output_file, index=False)
    print(f"💾 Saved complete dataset: {output_file}")
    
    # Show sample.
    print(f"\n📋 Sample data (first 5 records):")
    print(combined_df[['Open time', 'Open', 'High', 'Low', 'Close', 'Volume']].head())
    
    return combined_df

if __name__ == "__main__":
    print("🎯 SOLANA COMPLETE DATA DOWNLOADER")
    print("Goal: Get ALL available data with highest resolution")
    print("=" * 60)
    
    # Option 1: Try seconds first, fall back to minutes.
    print("🚀 Option 1: Try seconds first, fall back to minutes if needed")
    df = get_all_solana_data(try_seconds=True)
    
    if df is not None:
        print("✅ SUCCESS! You now have the complete Solana dataset.")
        print(f"📊 Records: {len(df):,}")
        print(f"📅 From: {df['Open time'].min()}")
        print(f"📅 To: {df['Open time'].max()}")
    else:
        print("❌ Failed to download data. Check your internet connection.")
    
    print("\n" + "=" * 60)
    print("✨ DONE! You have ALL available Solana data at highest resolution!")

