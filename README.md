# OpenStock

![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)

OpenStock is a CLI Inventory Management Suite written in Rust. It allows you to manage, track, and analyze inventory items efficiently from the command line.

## Features
- Add, update, delete, and list inventory items
- Import/export inventory from CSV files
- Filter and search inventory with flexible queries
- View detailed inventory statistics
- Performance monitoring and reporting (reports saved in `logs/`)
- Plugin support for extensibility
- Output in JSON, CSV, or table formats

## Installation (Linux/macOS, tar.gz)

1. **Download the latest release:**
   - Go to the [Releases](https://github.com/yourusername/OpenStock/releases) page and download the latest `openinv-x.y.z-linux.tar.gz` archive for your platform.

2. **Extract the archive:**
   ```sh
   tar -xzf openinv-x.y.z-linux.tar.gz
   ```

3. **Move the binary to a directory in your PATH:**
   ```sh
   sudo cp openinv /usr/local/bin/
   sudo chmod +x /usr/local/bin/openinv
   ```

4. **Verify installation:**
   ```sh
   openinv --help
   ```
   If you see `openinv: command not found`, ensure `/usr/local/bin` is in your PATH.

## Usage

### Add an item
```sh
<<<<<<< HEAD
openinv add-item --title "Widget" --price 9.99 --quantity 10 --category "Gadgets" --condition "New"
=======
openinv add --title "Widget" --price 9.99 --quantity 10 --category "Gadgets" --condition "New"
>>>>>>> 4c6ae46 (Shorten command names, update README and packaging, and improve install instructions)
```

### Import from CSV
```sh
openinv import --file items.csv
```

### List inventory (as table)
```sh
<<<<<<< HEAD
openinv list-inventory --format table
=======
openinv list --format table
>>>>>>> 4c6ae46 (Shorten command names, update README and packaging, and improve install instructions)
```

### List inventory (as JSON)
```sh
<<<<<<< HEAD
openinv list-inventory --format json
=======
openinv list --format json
>>>>>>> 4c6ae46 (Shorten command names, update README and packaging, and improve install instructions)
```

### Filter inventory by price and category
```sh
openinv filter --price 10-50 --category "Gadgets" --format json
```

### Filter inventory by brand and condition
```sh
openinv filter --brand "Acme" --condition "Used" --format table
```

### View statistics (table)
```sh
openinv stats --format table
```

### View statistics (JSON)
```sh
openinv stats --format json
```

### Validate a CSV file
```sh
openinv validate --file items.csv
```

## Advanced Usage

### Batch update items from CSV
```sh
openinv update --file updates.csv
```

### Export inventory to CSV
```sh
<<<<<<< HEAD
openinv list-inventory --format csv > export.csv
=======
openinv list --format csv > export.csv
>>>>>>> 4c6ae46 (Shorten command names, update README and packaging, and improve install instructions)
```

### Use a plugin (example: export to a custom platform)
```sh
openinv plugins run --name custom_export --args "platform=Shopify"
```

### Customize output fields
```sh
openinv filter --fields "item_id,title,price,brand" --format table
```

### Run with verbose performance metrics
```sh
openinv stats --format json --verbose
```

### Chain commands with shell scripting
```sh
openinv import --file items.csv && openinv stats --format table
```

## Performance Reports
Performance metrics are automatically saved as JSON files in the `logs/` directory after running `stats` or `filter` commands. Each report is timestamped for easy tracking.

## Troubleshooting & FAQ

- **Where are performance reports saved?**
  - In the `logs/` directory, as JSON files.
- **How do I add a new command?**
  - See the `src/commands/` directory for examples. Add your command and register it in `main.rs`.
- **Database errors?**
  - Ensure you have write permissions in the working directory. The SQLite database is created automatically.
- **How do I reset the database?**
  - Delete the database file (default: `inventory.db`) and rerun the CLI.

## Community & Contact
- Issues and feature requests: [GitHub Issues](https://github.com/yourusername/OpenStock/issues)
- Pull requests welcome! See [CONTRIBUTING](CONTRIBUTING.md) if available.
- Contact: your.email@example.com

## Contribution
Contributions are welcome! Please open issues or submit pull requests. See `Deployment.md` and `DeploymentChecklist.md` for deployment and development guidelines.

## License
<<<<<<< HEAD
MIT License 
=======
MIT License 
>>>>>>> 4c6ae46 (Shorten command names, update README and packaging, and improve install instructions)
