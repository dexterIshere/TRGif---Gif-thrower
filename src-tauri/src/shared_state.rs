use std::sync::{atomic::AtomicBool, Arc};

pub fn _get_shared_state() -> Result<Arc<AtomicBool>, String> {
    // keys
    let running = Arc::new(AtomicBool::new(true));
    Ok(running)
}
