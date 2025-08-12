use decorator::{hook, on_ok, on_result, retry, timeout};
use std::sync::{Arc, Mutex};
mod common;
use common::*;

#[tokio::test]
async fn test_async_on_ok_decorator_with_ok_result() {
    reset_callbacks();

    #[on_ok("async_on_ok_callback")]
    async fn async_function_that_returns_ok() -> Result<(), ()> {
        Ok(())
    }

    let _ = async_function_that_returns_ok().await;
    assert!(OK_CALLED.with(|c| *c.borrow()));
}

#[tokio::test]
async fn test_async_on_result_decorator_with_ok_result() {
    reset_callbacks();

    #[on_result(on_ok = "async_on_ok_callback", on_err = "async_on_err_callback")]
    async fn async_function_that_returns_ok() -> Result<(), ()> {
        Ok(())
    }

    let _ = async_function_that_returns_ok().await;
    assert!(OK_CALLED.with(|c| *c.borrow()));
    assert!(!ERR_CALLED.with(|c| *c.borrow()));
}

#[tokio::test]
async fn test_async_on_result_decorator_with_err_result() {
    reset_callbacks();

    #[on_result(on_ok = "async_on_ok_callback", on_err = "async_on_err_callback")]
    async fn async_function_that_returns_err() -> Result<(), ()> {
        Err(())
    }

    let _ = async_function_that_returns_err().await;
    assert!(!OK_CALLED.with(|c| *c.borrow()));
    assert!(ERR_CALLED.with(|c| *c.borrow()));
}

#[tokio::test]
async fn test_async_retry_decorator() {
    let retry_count = Arc::new(Mutex::new(0));

    #[retry(times = 3, delay_ms = 10)]
    async fn async_function_that_fails_twice(retry_count: Arc<Mutex<u32>>) -> Result<(), ()> {
        let mut count = retry_count.lock().unwrap();
        *count += 1;
        if *count < 3 {
            Err(())
        } else {
            Ok(())
        }
    }

    let result = async_function_that_fails_twice(retry_count.clone()).await;
    assert!(result.is_ok());
    assert_eq!(*retry_count.lock().unwrap(), 3);
}

#[tokio::test]
async fn test_async_retry_decorator_that_always_fails() {
    let retry_count = Arc::new(Mutex::new(0));

    #[retry(times = 3, delay_ms = 10)]
    async fn async_function_that_always_fails(retry_count: Arc<Mutex<u32>>) -> Result<(), ()> {
        let mut count = retry_count.lock().unwrap();
        *count += 1;
        Err(())
    }

    let result = async_function_that_always_fails(retry_count.clone()).await;
    assert!(result.is_err());
    assert_eq!(*retry_count.lock().unwrap(), 4);
}

#[tokio::test]
async fn test_async_timeout_decorator_success() {
    #[timeout(duration_ms = 100)]
    async fn async_function_that_finishes_quickly() -> Result<(), String> {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(())
    }

    let result = async_function_that_finishes_quickly().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_timeout_decorator_timeout() {
    #[timeout(duration_ms = 10)]
    async fn async_function_that_takes_too_long() -> Result<(), String> {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(())
    }

    let result = async_function_that_takes_too_long().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_hook_decorator() {
    reset_callbacks();

    #[hook(on_pre = "async_pre_hook_callback", on_post = "async_post_hook_callback")]
    async fn async_function_with_hooks() -> Result<(), ()> {
        assert!(PRE_HOOK_CALLED.with(|c| *c.borrow()));
        assert!(!POST_HOOK_CALLED.with(|c| *c.borrow()));
        Ok(())
    }

    let _ = async_function_with_hooks().await;
    assert!(PRE_HOOK_CALLED.with(|c| *c.borrow()));
    assert!(POST_HOOK_CALLED.with(|c| *c.borrow()));
}
