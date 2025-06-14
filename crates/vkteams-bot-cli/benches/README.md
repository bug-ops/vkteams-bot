# VK Teams Bot CLI Benchmarks

This directory contains performance benchmarks for the VK Teams Bot CLI application.

## Available Benchmarks

### 1. CLI Performance (`cli_performance.rs`)

Benchmarks core CLI functionality:

- **CLI Parsing**: Command-line argument parsing performance
- **Config Operations**: Configuration loading, validation, and serialization
- **Command Validation**: Input validation for different command types
- **String Operations**: Message formatting and JSON serialization
- **Output Formatting**: Pretty vs compact JSON formatting
- **Batch Operations**: Processing multiple messages/operations
- **Async Runtime**: Tokio runtime performance

### 2. Config Performance (`config_performance.rs`)

Focused benchmarks for configuration management:

- **Config Loading**: Default vs full configuration creation
- **Serialization**: TOML vs JSON serialization/deserialization
- **File Operations**: Reading and writing configuration files
- **Validation**: Configuration validation logic
- **Merging**: Configuration merging operations
- **Environment Variables**: Environment variable parsing
- **Caching**: Configuration cloning and hashing for cache operations

### 3. Specialized CLI Performance (`specialized_cli_performance.rs`)

Benchmarks for specific CLI operations:

- **Command Execution**: Simulation of different CLI commands
- **Validation Functions**: Chat ID and message validation
- **Progress Tracking**: Progress bar and tracker operations
- **Scheduler Operations**: Task creation and cron validation
- **Output Formatting**: Different data sizes and formats
- **File Operations**: File path validation and temp file generation

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark

```bash
# CLI performance only
cargo bench --bench cli_performance

# Config performance only  
cargo bench --bench config_performance

# Specialized CLI performance only
cargo bench --bench specialized_cli_performance
```

### Run with HTML Reports

```bash
cargo bench -- --output-format html
```

### Filter Specific Tests

```bash
# Run only parsing benchmarks
cargo bench parsing

# Run only validation benchmarks  
cargo bench validation

# Run only serialization benchmarks
cargo bench serialization
```

## Benchmark Results

Results are generated in `target/criterion/` directory:

- HTML reports: `target/criterion/report/index.html`
- Individual benchmark data in respective subdirectories

## Performance Metrics

The benchmarks measure:

- **Throughput**: Operations per second
- **Latency**: Time per operation
- **Memory Usage**: Memory allocation patterns
- **Scalability**: Performance with different data sizes

## Interpreting Results

- **Lower is better** for latency measurements
- **Higher is better** for throughput measurements
- Look for performance regressions between versions
- Pay attention to variance in measurements

## Contributing

When adding new benchmarks:

1. Place them in the appropriate benchmark file based on functionality
2. Use descriptive benchmark names
3. Include both small and large data sets where applicable
4. Use `black_box()` to prevent compiler optimizations
5. Document what the benchmark measures

## Benchmark Guidelines

- Use realistic data sizes and patterns
- Avoid benchmarking I/O operations (use mocks instead)
- Include both best-case and worst-case scenarios
- Test with different input variations
- Use `Throughput` for batch operations
- Group related benchmarks together
