# AI Video Editor - Common Commands

# Run the GUI
gui:
    cargo run -- --gui

# Run CLI with a file
cli input output:
    cargo run --release -- -i {{input}} -o {{output}}

# Build release (includes both CLI and GUI)
build:
    cargo build --release

# Build CLI only (smaller binary)
build-cli:
    cargo build --release --no-default-features --features cli

# Run tests
test:
    cargo test --lib

# Generate default config
config:
    cargo run --release -- --generate-config > ai-vid-editor.toml

# Watch a folder
watch input output:
    cargo run --release -- --watch {{input}} -O {{output}}

# Dry run (preview without processing)
dry input:
    cargo run --release -- -i {{input}} --dry-run

# Clean build artifacts
clean:
    cargo clean

# Check for issues
check:
    cargo check --all-features

# Format code
fmt:
    cargo fmt

# Lint
lint:
    cargo clippy --all-features
