# decorator-lib-rs

A Rust procedural macro library providing useful decorators for functions that return `Result` types.

## Decorators

- `#[on_ok("callback_fn")]`: Executes `callback_fn` if the decorated function returns `Ok`.
- `#[on_result(on_ok = "callback_ok", on_err = "callback_err")]`: Executes `callback_ok` on `Ok` or `callback_err` on `Err`.
- `#[retry(times = N, delay_ms = M)]`: Retries the function `N` times on `Err`, with an optional `M` ms delay.
- `#[timeout(duration_ms = D)]`: Returns an `Err` if the function doesn't complete within `D` ms.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
decorator = "25.7.0"
```

Example:

```rust
use decorator::{on_ok, retry};

fn my_callback() {
    println!("Operation successful!");
}

#[on_ok("my_callback")]
fn perform_operation() -> Result<(), String> {
    // ... some operation
    Ok(())
}

#[retry(times = 3, delay_ms = 100)]
fn flaky_operation() -> Result<(), String> {
    // ... might fail
    Err("Failed".to_string())
}
```
