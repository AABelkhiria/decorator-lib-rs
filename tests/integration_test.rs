use decorator::{hook, on_ok, on_result, retry, timeout};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

thread_local! {
    static OK_CALLED: RefCell<bool> = RefCell::new(false);
    static ERR_CALLED: RefCell<bool> = RefCell::new(false);
    static RETRY_COUNT: RefCell<u32> = RefCell::new(0);
    static PRE_HOOK_CALLED: RefCell<bool> = RefCell::new(false);
    static POST_HOOK_CALLED: RefCell<bool> = RefCell::new(false);
}

fn on_ok_callback() {
    OK_CALLED.with(|c| *c.borrow_mut() = true);
}

fn on_err_callback() {
    ERR_CALLED.with(|c| *c.borrow_mut() = true);
}

fn pre_hook_callback() {
    PRE_HOOK_CALLED.with(|c| *c.borrow_mut() = true);
}

fn post_hook_callback() {
    POST_HOOK_CALLED.with(|c| *c.borrow_mut() = true);
}

fn reset_callbacks() {
    OK_CALLED.with(|c| *c.borrow_mut() = false);
    ERR_CALLED.with(|c| *c.borrow_mut() = false);
    RETRY_COUNT.with(|c| *c.borrow_mut() = 0);
    PRE_HOOK_CALLED.with(|c| *c.borrow_mut() = false);
    POST_HOOK_CALLED.with(|c| *c.borrow_mut() = false);
}

#[test]
fn test_on_ok_decorator_with_ok_result() {
    reset_callbacks();

    #[on_ok("on_ok_callback")]
    fn function_that_returns_ok() -> Result<(), ()> {
        Ok(())
    }

    let _ = function_that_returns_ok();
    assert!(OK_CALLED.with(|c| *c.borrow()));
}

#[test]
fn test_on_ok_decorator_with_err_result() {
    reset_callbacks();

    #[on_ok("on_ok_callback")]
    fn function_that_returns_err() -> Result<(), ()> {
        Err(())
    }

    let _ = function_that_returns_err();
    assert!(!OK_CALLED.with(|c| *c.borrow()));
}

#[test]
fn test_on_result_decorator_with_ok_result() {
    reset_callbacks();

    #[on_result(on_ok = "on_ok_callback", on_err = "on_err_callback")]
    fn function_that_returns_ok() -> Result<(), ()> {
        Ok(())
    }

    let _ = function_that_returns_ok();
    assert!(OK_CALLED.with(|c| *c.borrow()));
    assert!(!ERR_CALLED.with(|c| *c.borrow()));
}

#[test]
fn test_on_result_decorator_with_err_result() {
    reset_callbacks();

    #[on_result(on_ok = "on_ok_callback", on_err = "on_err_callback")]
    fn function_that_returns_err() -> Result<(), ()> {
        Err(())
    }

    let _ = function_that_returns_err();
    assert!(!OK_CALLED.with(|c| *c.borrow()));
    assert!(ERR_CALLED.with(|c| *c.borrow()));
}

#[test]
fn test_on_result_decorator_with_missing_on_ok() {
    reset_callbacks();

    #[on_result(on_err = "on_err_callback")]
    fn function_that_returns_ok() -> Result<(), ()> {
        Ok(())
    }

    let _ = function_that_returns_ok();
    assert!(!OK_CALLED.with(|c| *c.borrow()));
    assert!(!ERR_CALLED.with(|c| *c.borrow()));
}

#[test]
fn test_on_result_decorator_with_missing_on_err() {
    reset_callbacks();

    #[on_result(on_ok = "on_ok_callback")]
    fn function_that_returns_err() -> Result<(), ()> {
        Err(())
    }

    let _ = function_that_returns_err();
    assert!(!OK_CALLED.with(|c| *c.borrow()));
    assert!(!ERR_CALLED.with(|c| *c.borrow()));
}

#[test]
fn test_retry_decorator() {
    reset_callbacks();

    #[retry(times = 3)]
    fn function_that_fails_twice() -> Result<(), ()> {
        RETRY_COUNT.with(|c| *c.borrow_mut() += 1);
        if RETRY_COUNT.with(|c| *c.borrow()) < 3 {
            Err(())
        } else {
            Ok(())
        }
    }

    let result = function_that_fails_twice();
    assert!(result.is_ok());
    assert_eq!(RETRY_COUNT.with(|c| *c.borrow()), 3);
}

#[test]
fn test_retry_decorator_that_always_fails() {
    reset_callbacks();

    #[retry(times = 3)]
    fn function_that_always_fails() -> Result<(), ()> {
        RETRY_COUNT.with(|c| *c.borrow_mut() += 1);
        Err(())
    }

    let result = function_that_always_fails();
    assert!(result.is_err());
    assert_eq!(RETRY_COUNT.with(|c| *c.borrow()), 4);
}

#[test]
fn test_retry_decorator_with_delay() {
    reset_callbacks();

    #[retry(times = 3, delay_ms = 10)]
    fn function_that_fails_twice() -> Result<(), ()> {
        RETRY_COUNT.with(|c| *c.borrow_mut() += 1);
        if RETRY_COUNT.with(|c| *c.borrow()) < 3 {
            Err(())
        } else {
            Ok(())
        }
    }

    let result = function_that_fails_twice();
    assert!(result.is_ok());
    assert_eq!(RETRY_COUNT.with(|c| *c.borrow()), 3);
}

#[test]
fn test_timeout_decorator_success() {
    #[timeout(duration_ms = 10)]
    fn function_that_finishes_quickly() -> Result<(), String> {
        Ok(())
    }

    let result = function_that_finishes_quickly();
    assert!(result.is_ok());
}

#[test]
fn test_timeout_decorator_timeout() {
    #[timeout(duration_ms = 10)]
    fn function_that_takes_too_long() -> Result<(), String> {
        std::thread::sleep(std::time::Duration::from_millis(50));
        Ok(())
    }

    let result = function_that_takes_too_long();
    assert!(result.is_err());
}

#[test]
fn test_hook_decorator() {
    reset_callbacks();

    #[hook(on_pre = "pre_hook_callback", on_post = "post_hook_callback")]
    fn function_with_hooks() -> Result<(), ()> {
        assert!(PRE_HOOK_CALLED.with(|c| *c.borrow()));
        assert!(!POST_HOOK_CALLED.with(|c| *c.borrow()));
        Ok(())
    }

    let _ = function_with_hooks();
    assert!(PRE_HOOK_CALLED.with(|c| *c.borrow()));
    assert!(POST_HOOK_CALLED.with(|c| *c.borrow()));
}

#[test]
fn test_hook_decorator_only_pre() {
    reset_callbacks();

    #[hook(on_pre = "pre_hook_callback")]
    fn function_with_only_pre_hook() -> Result<(), ()> {
        assert!(PRE_HOOK_CALLED.with(|c| *c.borrow()));
        assert!(!POST_HOOK_CALLED.with(|c| *c.borrow()));
        Ok(())
    }

    let _ = function_with_only_pre_hook();
    assert!(PRE_HOOK_CALLED.with(|c| *c.borrow()));
    assert!(!POST_HOOK_CALLED.with(|c| *c.borrow()));
}

#[test]
fn test_hook_decorator_only_post() {
    reset_callbacks();

    #[hook(on_post = "post_hook_callback")]
    fn function_with_only_post_hook() -> Result<(), ()> {
        assert!(!PRE_HOOK_CALLED.with(|c| *c.borrow()));
        assert!(!POST_HOOK_CALLED.with(|c| *c.borrow()));
        Ok(())
    }

    let _ = function_with_only_post_hook();
    assert!(!PRE_HOOK_CALLED.with(|c| *c.borrow()));
    assert!(POST_HOOK_CALLED.with(|c| *c.borrow()));
}

// Async tests
async fn async_on_ok_callback() {
    OK_CALLED.with(|c| *c.borrow_mut() = true);
}

async fn async_on_err_callback() {
    ERR_CALLED.with(|c| *c.borrow_mut() = true);
}

async fn async_pre_hook_callback() {
    PRE_HOOK_CALLED.with(|c| *c.borrow_mut() = true);
}

async fn async_post_hook_callback() {
    POST_HOOK_CALLED.with(|c| *c.borrow_mut() = true);
}

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