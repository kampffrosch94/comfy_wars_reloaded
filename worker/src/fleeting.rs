use cosync::CosyncInput;

use crate::persistent::PersistentState;

/// dropped and recreated on reload
/// you can change this definition without breaking hotreloading
pub struct FleetingState {
    pub queue: cosync::Cosync<PersistentState>,
}

impl FleetingState {
    pub fn new() -> Self {
        let mut queue = cosync::Cosync::new();
        queue.queue(move |mut input: CosyncInput<PersistentState>| async move {
            for _ in 0..5 {
                cosync::sleep_ticks(30).await;
                let mut _s = input.get();
            }
        });
        Self { queue }
    }
}
