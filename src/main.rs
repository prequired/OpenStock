use clap::{Parser, Subcommand, ValueEnum};
use anyhow::Result;
use std::sync::Arc;
use chrono;

mod commands;
mod db;
mod validation;
mod plugins;
mod config;
mod logging;
mod output;
mod error;

use commands::{
    add::handle_add_item,
    update::{Update, execute as handle_update},
    delete::handle_delete_item,
    list::{handle_list_inventory, OutputFormat},
    import::handle_import,
    filter::handle_filter,
    migrate::handle_migrate,
    help::handle_help,
    commands::handle_commands,
    stats::handle_stats,
    validate::handle_validate,
};
use db::schema::initialize_database;
use config::optimization::{PerformanceMonitor, QueryCache, optimize_database};

#[derive(Parser)]
#[command(name = "inventory")]
#[command(about = "CLI Inventory Management Suite")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Output format (json, csv, table)
    #[arg(long, value_enum, default_value = "json")]
    format: Option<OutputFormat>,
    
    /// Enable verbose output with performance metrics
    #[arg(long)]
    verbose: bool,
    
    /// Log level
    #[arg(long, value_enum, default_value = "info")]
    log_level: Option<LogLevel>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item to inventory
    Add {
        /// Item title
        #[arg(short, long)]
        title: String,
        
        /// Item price
        #[arg(short, long)]
        price: f64,
        
        /// Item quantity
        #[arg(short, long)]
        quantity: i32,
        
        /// Item category
        #[arg(short, long)]
        category: String,
        
        /// Item condition
        #[arg(short = 'n', long)]
        condition: String,
        
        /// Item brand
        #[arg(short, long)]
        brand: Option<String>,
        
        /// Item description
        #[arg(short, long)]
        description: Option<String>,
        
        /// UPC code
        #[arg(long)]
        upc: Option<String>,
        
        /// Item size
        #[arg(long)]
        size: Option<String>,
        
        /// Original price
        #[arg(long)]
        original_price: Option<f64>,
        
        /// Hashtags
        #[arg(long)]
        hashtags: Option<String>,
        
        /// Colorway
        #[arg(long)]
        colorway: Option<String>,
        
        /// Release date
        #[arg(long)]
        release_date: Option<String>,
        
        /// Internal notes
        #[arg(long)]
        internal_notes: Option<String>,
    },
    
    /// Update items from CSV file
    Update(Update),
    
    /// Delete an item by ID
    Delete {
        /// Item ID to delete
        #[arg(short, long)]
        id: i32,
    },
    
    /// List inventory items
    List {
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,
    },
    
    /// Import items from CSV file
    Import {
        /// CSV file to import
        #[arg(short, long)]
        file: String,
    },
    
    /// Filter inventory items
    Filter {
        /// Price range (e.g., 10-50)
        #[arg(short, long)]
        price: Option<String>,
        
        /// Category filter
        #[arg(short, long)]
        category: Option<String>,
        
        /// Condition filter
        #[arg(short = 'n', long)]
        condition: Option<String>,
        
        /// Brand filter
        #[arg(short, long)]
        brand: Option<String>,
        
        /// Fields to display
        #[arg(short = 'l', long)]
        fields: Option<String>,
        
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,
    },
    
    /// Run database migrations
    Migrate,
    
    /// Show field shortcuts
    Fields,
    
    /// List available commands
    Commands,
    
    /// Show inventory statistics
    Stats {
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,
    },
    
    /// Validate a CSV file against the inventory schema
    Validate {
        /// CSV file to validate
        #[arg(short, long)]
        file: String,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize database connection
    let conn = initialize_database(None)?;
    
    // Initialize performance optimizations
    optimize_database(&conn)?;
    let monitor = Arc::new(PerformanceMonitor::new());
    let cache = Arc::new(QueryCache::new(monitor.clone()));
    
    // TODO: Initialize logging and config
    // TODO: Handle subcommands
    
    match cli.command {
        Commands::Add { title, price, quantity, category, condition, brand, description, upc, size, original_price, hashtags, colorway, release_date, internal_notes } => {
            handle_add_item(title, price, quantity, category, condition, brand, description, upc, size, original_price, hashtags, colorway, release_date, internal_notes)
        }
        Commands::Update(args) => {
            handle_update(args, &conn)
        }
        Commands::Delete { id } => {
            handle_delete_item(id, &conn)
        }
        Commands::List { format } => {
            handle_list_inventory(&conn, format)
        }
        Commands::Import { file } => {
            handle_import(file, &conn, false)
        }
        Commands::Filter { price, category, condition, brand, fields, format } => {
            let result = handle_filter(&conn, price, category, condition, brand, fields, format, Some(monitor.clone()), Some(cache.clone()));
            // Save performance report
            let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S");
            let filename = format!("performance_filter_{}.json", timestamp);
            if let Err(e) = monitor.write_performance_report(&filename) {
                eprintln!("Failed to write performance report: {}", e);
            }
            result
        }
        Commands::Migrate => {
            handle_migrate()
        }
        Commands::Fields => {
            handle_help()
        }
        Commands::Commands => {
            handle_commands()
        }
        Commands::Stats { format } => {
            let result = handle_stats(&conn, format, Some(monitor.clone()), Some(cache.clone()));
            // Save performance report
            let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S");
            let filename = format!("performance_stats_{}.json", timestamp);
            if let Err(e) = monitor.write_performance_report(&filename) {
                eprintln!("Failed to write performance report: {}", e);
            }
            result
        }
        Commands::Validate { file } => {
            match handle_validate(&file) {
                Ok(()) => Ok(()),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
    }
} 