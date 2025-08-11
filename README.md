# decorator-lib-rs

A Rust procedural macro library providing useful decorators for functions that return `Result` types.

## Decorators

-   Executes `callback_fn` if the decorated function returns `Ok`.
    ```rust
    #[on_ok("callback_fn")]
    ```
    
-   Executes `callback_ok` on `Ok` or `callback_err` on `Err`.
    ```rust
    #[on_result(on_ok = "callback_ok", on_err = "callback_err")]
    ```
-   Retries the function `N` times on `Err`, with an optional `M` ms delay.
    ```rust
    #[retry(times = N, delay_ms = M)]
    ```
-   Returns an `Err` if the function doesn't complete within `D` ms.
    ```rust
    #[timeout(duration_ms = D)]
    ```
-   Executes `on_pre` before the function and `on_post` after it.
    ```rust
    #[hook(on_pre = "pre_hook_fn", on_post = "post_hook_fn")]
    ```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
decorator = "0.0.2"
## Or from GitHub
decorator = { git = "https://github.com/AABelkhiria/decorator-lib-rs.git", branch = "main" }
```

## Registry

This repository also serves as a crate registry. To use it, add the following to your `.cargo/config.toml`:

```toml
[registries.ash-registry]
index = "https://github.com/AABelkhiria/ash-registry"

[net]
git-fetch-with-cli = true
```

Then, you can add the crate to your `Cargo.toml` like this:

```toml
[dependencies]
decorator-lib-rs = { version = "0.0.2", registry = "ash-registry" }
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
