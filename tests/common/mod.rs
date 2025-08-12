use std::cell::RefCell;

thread_local! {
    pub static OK_CALLED: RefCell<bool> = RefCell::new(false);
    pub static ERR_CALLED: RefCell<bool> = RefCell::new(false);
    pub static RETRY_COUNT: RefCell<u32> = RefCell::new(0);
    pub static PRE_HOOK_CALLED: RefCell<bool> = RefCell::new(false);
    pub static POST_HOOK_CALLED: RefCell<bool> = RefCell::new(false);
}

pub fn on_ok_callback() {
    OK_CALLED.with(|c| *c.borrow_mut() = true);
}

pub fn on_err_callback() {
    ERR_CALLED.with(|c| *c.borrow_mut() = true);
}

pub fn pre_hook_callback() {
    PRE_HOOK_CALLED.with(|c| *c.borrow_mut() = true);
}

pub fn post_hook_callback() {
    POST_HOOK_CALLED.with(|c| *c.borrow_mut() = true);
}

pub fn reset_callbacks() {
    OK_CALLED.with(|c| *c.borrow_mut() = false);
    ERR_CALLED.with(|c| *c.borrow_mut() = false);
    RETRY_COUNT.with(|c| *c.borrow_mut() = 0);
    PRE_HOOK_CALLED.with(|c| *c.borrow_mut() = false);
    POST_HOOK_CALLED.with(|c| *c.borrow_mut() = false);
}

pub async fn async_on_ok_callback() {
    OK_CALLED.with(|c| *c.borrow_mut() = true);
}

pub async fn async_on_err_callback() {
    ERR_CALLED.with(|c| *c.borrow_mut() = true);
}

pub async fn async_pre_hook_callback() {
    PRE_HOOK_CALLED.with(|c| *c.borrow_mut() = true);
}

pub async fn async_post_hook_callback() {
    POST_HOOK_CALLED.with(|c| *c.borrow_mut() = true);
}
