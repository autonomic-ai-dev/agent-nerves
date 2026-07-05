//! WASM event filters (allow/deny) for broker messages.

use agent_body_core::default_fuel_limit;
use anyhow::{Context, Result};
use std::path::Path;
use wasmtime::{Instance, Store};

/// Evaluate a WASM filter module against a payload using the shared module cache.
///
/// Contract: module exports `memory` and `filter(ptr: i32, len: i32) -> i32`
/// where non-zero means allow and zero means drop.
pub fn evaluate_wasm(path: &Path, payload: &[u8]) -> Result<bool> {
    let wasm_bytes =
        std::fs::read(path).with_context(|| format!("read wasm filter {}", path.display()))?;
    evaluate_wasm_bytes(&wasm_bytes, payload)
        .with_context(|| format!("evaluate wasm filter {}", path.display()))
}

pub fn evaluate_wasm_bytes(wasm_bytes: &[u8], payload: &[u8]) -> Result<bool> {
    let shared = agent_body_core::wasm_engine::shared();
    let module = shared.get_or_compile(wasm_bytes)?;
    let mut store = Store::new(shared.engine(), ());
    store
        .set_fuel(default_fuel_limit())
        .context("set wasm fuel budget")?;
    let instance =
        Instance::new(&mut store, &module, &[]).context("instantiate wasm filter module")?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .context("wasm filter must export memory")?;
    let filter = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "filter")
        .context("wasm filter must export filter(i32,i32)->i32")?;

    let mem = memory.data_mut(&mut store);
    if payload.len() > mem.len() {
        anyhow::bail!(
            "payload {} bytes exceeds wasm memory {} bytes",
            payload.len(),
            mem.len()
        );
    }
    mem[..payload.len()].copy_from_slice(payload);
    let decision = filter.call(&mut store, (0, payload.len() as i32))?;
    Ok(decision != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALLOW_WAT: &str = r#"
    (module
      (memory (export "memory") 1)
      (func (export "filter") (param i32 i32) (result i32)
        i32.const 1)
    )
    "#;

    fn allow_wasm_bytes() -> Vec<u8> {
        wat::parse_str(ALLOW_WAT).expect("parse allow filter wat")
    }

    #[test]
    fn cached_filter_allows_payload() {
        let wasm = allow_wasm_bytes();
        let shared = agent_body_core::wasm_engine::shared();
        shared.clear_cache();
        assert!(evaluate_wasm_bytes(&wasm, b"{}").unwrap());
        assert_eq!(shared.cache_len(), 1);
        assert!(evaluate_wasm_bytes(&wasm, b"{}").unwrap());
    }
}
