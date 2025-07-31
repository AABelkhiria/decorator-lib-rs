use decorator::{hook, on_ok, on_result, retry, timeout};
use std::cell::RefCell;

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
    assert_eq!(result.unwrap_err(), "Function timed out after 10ms");
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
