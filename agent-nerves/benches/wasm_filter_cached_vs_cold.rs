use std::hint::black_box;

use agent_nerves::wasm_filter;

const ALLOW_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (func (export "filter") (param i32 i32) (result i32)
    i32.const 1)
)
"#;

fn allow_wasm() -> Vec<u8> {
    wat::parse_str(ALLOW_WAT).expect("parse bench wasm")
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let filter_name = args
        .iter()
        .position(|a| a == "--")
        .map(|i| i + 1)
        .and_then(|i| args.get(i))
        .map(String::as_str)
        .unwrap_or("");

    if !filter_name.is_empty() && filter_name != "wasm_filter_cached_vs_cold" {
        eprintln!("unknown bench filter: {filter_name}");
        std::process::exit(1);
    }

    let wasm = allow_wasm();
    let payload = br#"{"event":"bench"}"#;
    let iters = 50u32;

    agent_body_core::wasm_engine::shared().clear_cache();

    let cold_start = std::time::Instant::now();
    for _ in 0..iters {
        agent_body_core::wasm_engine::shared().clear_cache();
        black_box(
            wasm_filter::evaluate_wasm_bytes(&wasm, payload)
                .expect("cached eval after cache clear"),
        );
    }
    let cold_elapsed = cold_start.elapsed();

    let cached_start = std::time::Instant::now();
    for _ in 0..iters {
        black_box(wasm_filter::evaluate_wasm_bytes(&wasm, payload).expect("cached eval"));
    }
    let cached_elapsed = cached_start.elapsed();

    println!("wasm_filter_cached_vs_cold:");
    println!("  iterations: {iters}");
    println!("  cold (cache cleared each iter): {cold_elapsed:?}");
    println!("  cached (warm module cache):     {cached_elapsed:?}");
    if cached_elapsed < cold_elapsed {
        println!(
            "  speedup: {:.2}x",
            cold_elapsed.as_secs_f64() / cached_elapsed.as_secs_f64()
        );
    }
}
