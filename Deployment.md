# Deployment Guide

## Overview
This guide explains how to build and run the CLI Inventory Management Suite using Docker and how to use the CI/CD pipeline with GitHub Actions.

## Prerequisites
- Docker (version 20.10 or higher)
- GitHub Actions (for CI/CD)
- Rust 1.82 (for local builds)
- SQLite3 (included in Docker image)

## Building the Docker Image
1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd inventory
   ```
2. Build the Docker image:
   ```bash
   docker build -t inventory-cli:latest .
   ```

## Running the CLI in Docker
- Run the CLI with a persistent volume for `~/.inventory`:
  ```bash
  docker run --rm -v $(pwd)/.inventory:/root/.inventory inventory-cli:latest --help
  ```
- Example commands:
  ```bash
  docker run --rm -v $(pwd)/.inventory:/root/.inventory inventory-cli:latest add-item --title "Test Item" --price 10.0 --quantity 5 --category sneakers --condition new
  docker run --rm -v $(pwd)/.inventory:/root/.inventory inventory-cli:latest filter --price 0-20 -f id,title,price
  docker run --rm -v $(pwd)/.inventory:/root/.inventory inventory-cli:latest stats --format json
  ```

## Environment Variables
- `HOME`: Set to `/root` inside the container. The `.inventory` directory (database, logs, exports) is stored here.
- Use a volume to persist data: `-v /path/to/local/.inventory:/root/.inventory`.

## CI/CD Pipeline
- The pipeline is defined in `.github/workflows/ci.yml`.
- **Triggers**: Runs on push or pull requests to the `main` branch.
- **Steps**:
  1. Checks out the code.
  2. Installs Rust 1.82.
  3. Builds the project (`cargo build --release`).
  4. Runs tests (`cargo test -- --nocapture`).
  5. Builds the Docker image.
  6. Runs sample CLI commands in the container.
  7. Stores test results and `.inventory` directory as artifacts.
- **Artifacts**:
  - Test results: `target/**/*.log`
  - Docker volume: `/tmp/.inventory`

## Local Development
- Build and test locally:
  ```bash
  cargo build --release
  cargo test -- --nocapture
  ```
- Run the CLI:
  ```bash
  ./target/release/inventory --help
  ```

## Notes
- The Docker image uses `debian:bookworm-slim` for minimal size.
- SQLite database and file outputs (logs, exports, failed files) are stored in `/root/.inventory`, mapped to a host volume.
- The CI/CD pipeline does not push to a registry by default. To enable, add a `docker push` step with credentials. 