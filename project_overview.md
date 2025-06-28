Comprehensive Specification for CLI Inventory Management Suite
Overview
This specification defines a Rust-based command-line interface (CLI) inventory management suite, adhering to Unix philosophy principles (small, modular, text-based, interoperable) for managing inventory compatible with eBay, Mercari, StockX, and Poshmark. The suite focuses on inventory management, with cross-listing features deferred to separate tools. It supports 20–100 listings, uses a single SQLite database, and emphasizes configurability, strict validation, and Unix tool interoperability.
Key Features

Single CLI Tool: Subcommands (inventory <subcommand>) for modular, Unix-friendly design.
SQLite Database: Single database with platform-specific fields, auto-created on first run.
Strict Validation: Platform-specific rules (e.g., eBay’s 80-character title limit) enforced at CLI input.
CSV Import/Export: Fixed schema, interactive prompts for invalid rows, JSON/CSV/table outputs.
Filtering: Simple syntax with configurable fields and shortcuts.
Backups: Automatic, TOML-configurable, timestamped, with user-configurable retention.
Logging: Structured, TOML-configurable, with rotation.
Plugins: Dynamically loaded shared libraries for platform-specific subcommands.
Performance Metrics: Structured plain text via --verbose.
TOML Configuration: Controls database, backups, logging, and plugins.

Data Model
SQLite Schema
The single SQLite database (~/.inventory/inventory.db) stores all inventory data with the following schema:
CREATE TABLE items (
    item_id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK(length(title) <= 80), -- eBay max length
    description TEXT,
    price REAL NOT NULL CHECK(price >= 0),
    quantity INTEGER NOT NULL CHECK(quantity >= 0),
    photos TEXT, -- Comma-separated file paths or URLs
    category TEXT NOT NULL,
    condition TEXT NOT NULL, -- e.g., new, used, deadstock
    brand TEXT,
    upc TEXT, -- Required for StockX
    item_specifics TEXT, -- JSON, e.g., {"size": "M", "color": "Red"}
    shipping_details TEXT, -- JSON, e.g., {"weight": "1kg", "carrier": "USPS"}
    size TEXT, -- Poshmark/StockX requirement
    original_price REAL, -- Poshmark optional
    hashtags TEXT, -- Poshmark/Mercari optional
    colorway TEXT, -- StockX optional
    release_date TEXT, -- StockX optional
    platform_status TEXT, -- JSON, e.g., {"ebay": true, "stockx": false}
    internal_notes TEXT,
    last_updated TEXT NOT NULL, -- ISO 8601, e.g., "2025-06-27T14:59:00Z"
    status TEXT NOT NULL CHECK(status IN ('active', 'sold', 'draft'))
);


Initialization: Auto-created on first run if missing.
Access: Single-user, sequential processing for 20–100 listings.
Migration: Supports schema updates via inventory migrate.
Error Handling: Exits with detailed error on corruption/inaccessibility (e.g., “Database ~/.inventory/inventory.db inaccessible: file locked”).

Validation

Rules: Strict, platform-specific validation at CLI input stage:
eBay: Title ≤ 80 chars, item specifics, shipping details.
StockX: Mandatory UPC, size, optional colorway, release date.
Poshmark: Size for clothing, optional original price, hashtags.
Mercari: Shipping options, optional hashtags.


Error Output: JSON format with metadata (field, message, row, value), e.g.:{"errors": [{"field": "title", "message": "Exceeds eBay's 80-character limit", "row": 5, "value": "Very long title..."}]}


Failed Inputs: Saved to ~/.inventory/failed/failed_<command>_<timestamp>.json, retained for 7 days (TOML-configurable).

CSV Import/Export

Schema: Fixed: item_id,title,description,price,quantity,upc,category,condition,brand.
Import:
Command: inventory import --file items.csv.
Interactive prompts for each invalid row, showing full row, requiring all fixes in one session.
Validates after all fixes for a row, applies immediately.
Failed inputs saved to temp file.


Export:
Supports JSON (default), CSV, plain text table via --format {json,csv,table}.
Example: inventory list-inventory --format csv.



Filter Command

Syntax: inventory filter --price 10-50 --category clothing --condition new -f id,title,price.
Fields: Configurable via -f id,title,price,quantity,condition,category,brand with shortcuts (e.g., id, t, p, q, c, cat, b).
Help: inventory help fields outputs plain text (e.g., id: item_id, t: title, p: price).
Output: JSON (default), CSV, or plain text table, no sorting (use sort externally).

Backups

Configuration: TOML-configurable (time-based, e.g., “daily”, or command count, e.g., “commands:10”).
Naming: Timestamped (e.g., inventory_2025-06-27T13:54:00.sql).
Directory: User-configurable, defaults to ~/.inventory/backups/, creates with 700 permissions.
Retention: 14 days, single yes/no prompt for deletion.
Error Handling: Falls back to default directory with warning if invalid, logs failures without pausing, attempts to create directory.

Output Formats

Formats: JSON (default), CSV, plain text table.
Plain Text Table:
Fields: item_id, title, price, quantity, condition, category, brand.
Separator: |.
Wraps long fields, truncates at 50 chars (title) or 30 (others) with ....
Example: 1 | Air Jordan 1... | 150.00 | 2 | new | sneakers | Nike.



Logging

File: ~/.inventory/logs/inventory.log, 600 permissions.
Crate: log4rs for structured logging.
Levels: Standard (error, warn, info, debug, trace), configurable via --log-level, resets to info, overridable in TOML.
Rotation: TOML-configurable (default: 10MB, 7-day retention).
Example: 2025-06-27T14:59:00Z INFO Added item_id=1.

Plugins

Directory: TOML-configurable (e.g., ~/.inventory/plugins/), fails with detailed error if invalid (e.g., “Directory ~/.inventory/plugins/ not found”).
Loading: Dynamically loaded shared libraries, skips invalid plugins with warnings (e.g., “Update plugin stockx.so to version 0.1.0”).
Feedback: Displays on startup (e.g., “Loaded plugins: ebay, stockx”).
Versioning: Enforces compatibility to prevent crashes.
Subcommands: Platform-specific (e.g., inventory stockx validate --id 1).

Batch Updates

Command: inventory update --file updates.csv.
Schema: Same as import CSV.
Errors: Generates failed_updates_<timestamp>.csv with error column (e.g., invalid_price), logs failures, notifies: “Failed rows saved to failed_updates_2025-06-27.csv. Retry with ‘inventory update --retry’.”
Retry: inventory update --retry failed_updates.csv, unlimited retries, interactive prompts.

TOML Configuration

File: ~/.inventory/config.toml.
Fields:[database]
path = "~/.inventory/inventory.db"

[backup]
enabled = true
directory = "~/.inventory/backups/"
interval = "daily" # or "commands:10"
retention_days = 14
failed_retention_days = 7

[logging]
level = "info"
path = "~/.inventory/logs/inventory.log"
rotation_size_mb = 10
rotation_retention_days = 7

[plugins]
directory = "~/.inventory/plugins/"


Validation: Exits with line/column-specific error if invalid (e.g., “Invalid TOML at line 10, column 5: missing field ‘path’”).
User-Defined Fields: Deferred storage.

Performance Metrics

Flag: --verbose.
Format: Structured plain text (e.g., Execution time: 0.5s, Rows processed: 50, Memory: 10MB, Query time: 0.1s), logged to file.
Metrics: Fixed set (execution time, rows processed, memory usage, query time).

CLI Command Flow

Subcommands:
inventory add-item --title "..." --price 10.99 --quantity 5 ...
inventory update --file updates.csv and inventory update --retry failed_updates.csv
inventory import --file items.csv
inventory filter --price 10-50 --category clothing -f id,title,price
inventory list-inventory --format table
inventory delete-item --id 1
inventory migrate
inventory commands
inventory help [fields]
Platform-specific (e.g., inventory stockx validate --id 1).


Flags: --help, -h, --format {json,csv,table}, --verbose, --log-level {error,warn,info,debug,trace}.
Help: --help and -h for each subcommand, inventory help fields for field shortcuts, inventory commands for dynamic subcommand listing.

Rust Project Structure
inventory/
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── commands/              # Subcommand logic
│   │   ├── add.rs             # `add-item`
│   │   ├── update.rs          # `update` and `update --retry`
│   │   ├── delete.rs          # `delete-item`
│   │   ├── list.rs            # `list-inventory`
│   │   ├── import.rs          # `import`
│   │   ├── filter.rs          # `filter`
│   │   ├── migrate.rs         # `migrate`
│   │   ├── help.rs            # `help` and `help fields`
│   │   └── commands.rs        # `commands`
│   ├── db/                    # SQLite logic
│   │   ├── schema.rs          # Schema and migration
│   │   └── queries.rs         # CRUD operations
│   ├── plugins/               # Plugin management
│   │   ├── loader.rs          # Dynamic loading and versioning
│   │   └── platforms/         # Platform-specific (e.g., ebay.rs, stockx.rs)
│   ├── config/                # TOML parsing
│   │   └── config.rs
│   ├── logging/               # Logging setup
│   │   └── logger.rs          # Log4rs configuration
│   ├── output/                # Output formatting
│   │   └── format.rs          # JSON, CSV, table rendering
│   └── error/                 # Error handling
│       └── error.rs
├── plugins/                   # Shared library plugins
├── Cargo.toml                 # Dependencies
└── README.md                  # Documentation

Cargo.toml
[package]
name = "inventory"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5", features = ["derive"] } # CLI parsing
rusqlite = { version = "0.31", features = ["bundled"] } # SQLite
serde = { version = "1.0", features = ["derive"] } # JSON/TOML
serde_json = "1.0" # JSON output
toml = "0.8" # TOML parsing
log4rs = "1.3" # Structured logging
csv = "1.3" # CSV import/export
libloading = "0.8" # Dynamic plugin loading
chrono = "0.4" # Timestamps
dirs = "5.0" # Home directory access
anyhow = "1.0" # Error handling

[profile.release]
opt-level = 3
strip = true

Setup Guide

Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
Clone Repository: git clone <repo_url> && cd inventory
Build: cargo build --release
Run: ./target/release/inventory --help
First Run: Creates ~/.inventory/, ~/.inventory/backups/, ~/.inventory/logs/, ~/.inventory/plugins/, ~/.inventory/failed/, and ~/.inventory/config.toml.
Plugins: Place *.so files in ~/.inventory/plugins/.
Usage Example:inventory add-item -t "Air Jordan 1" -p 150.00 -q 2 -c new -cat sneakers -b Nike --upc 123456789012
inventory filter -p 10-50 -cat clothing -f id,t,p --format table
inventory import --file items.csv



Error Handling

Validation: JSON errors with platform-specific details.
TOML: Exits with line/column error.
Database: Exits with detailed error.
Plugins: Skips invalid plugins, warns with version suggestions.
Backups: Logs failures, continues operation.

Extensibility

Plugins: Add new platforms via shared libraries.
Schema: inventory migrate for future fields.
Subcommands: Dynamic listing via inventory commands.
