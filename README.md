# Discord User Analyzer

> A high-performance Rust tool for analyzing JSON message data to identify patterns in communications with deleted users.

## Overview

The Deleted User Analyzer is a command-line utility designed to efficiently process large JSON datasets containing message histories. It identifies messages sent to deleted users and provides comprehensive analytics including word frequency analysis, author statistics, and communication patterns.


## Installation

Ensure you have Rust installed on your system. If not, visit [rustup.rs](https://rustup.rs/) to install Rust.

```bash
git clone <repository-url>
cd parse
cargo build --release
```

## Usage

### Basic Usage

```bash
cargo run -- --input sample_data.json
```

### Advanced Usage

```bash
# Save results to JSON file
cargo run -- --input data.json --output results.json

# Enable verbose output with detailed statistics
cargo run -- --input data.json --verbose

# Set minimum word length for analysis
cargo run -- --input data.json --min-word-length 4
```

### Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--input` | `-i` | Input JSON file path | Required |
| `--output` | `-o` | Output JSON file path | Optional |
| `--verbose` | `-v` | Enable detailed output | false |
| `--min-word-length` | | Minimum word length for analysis | 3 |

## Input Format

The tool expects JSON data with the following message structure:

```json
{
  "message_id": "string",
  "content": "string",
  "timestamp": "string",
  "author_name": "string",
  "author_nickname": "string",
  "author_id": "string",
  "mentioned_user_name": "string (optional)",
  "mentioned_user_nickname": "string (optional)"
}
```



## Performance

Typical performance on modern hardware:
- 100K messages: < 1 second
- 1M messages: < 10 seconds
- 10M messages: < 2 minutes

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | JSON serialization/deserialization |
| `serde_json` | 1.0 | JSON parsing |
| `clap` | 4.0 | Command-line argument parsing |
| `anyhow` | 1.0 | Error handling |
| `rayon` | 1.7 | Parallel processing |
| `hashbrown` | 0.14 | High-performance hash maps |

## Testing

Run the test suite to ensure everything works correctly:

```bash
cargo test
```

The test  includes:
- Content tokenization validation
- Word filtering accuracy
- Minimum length enforcement
- Number handling in content


## License

This project is available under standard open source licensing terms.

---

**Built with Rust** | **Optimized for Performance** | 
