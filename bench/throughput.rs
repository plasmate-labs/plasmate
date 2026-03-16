// Throughput benchmark: 100 pages from local server
// Matches Lightpanda's benchmark methodology exactly.
//
// Build: cargo build --release
// Run:   python3 bench/local_server.py 8765 &
//        cargo run --release -- throughput-bench

// This is integrated into main.rs as a subcommand.
