//! WASM event filters (allow/deny) for broker messages.

use anyhow::{Context, Result};
use std::path::Path;
use wasmtime::{Engine, Instance, Module, Store};

/// Evaluate a WASM filter module against a payload.
///
/// Contract: module exports `memory` and `filter(ptr: i32, len: i32) -> i32`
/// where non-zero means allow and zero means drop.
pub fn evaluate_wasm(path: &Path, payload: &[u8]) -> Result<bool> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, path)
        .with_context(|| format!("load wasm filter {}", path.display()))?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])
        .with_context(|| format!("instantiate wasm filter {}", path.display()))?;

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
