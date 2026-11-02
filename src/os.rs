//! Initial BlueOS pump responsibilities. Clock, IO, tick and shutdown portions
//! remain pending in the source correspondence.
#![allow(non_snake_case)]
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct BlueOS {
    mInsidePump: AtomicBool,
}

struct PumpGuard<'a>(&'a AtomicBool);
impl Drop for PumpGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl BlueOS {
    pub fn PumpOS(&self) {
        self.PumpOSInternal();
    }
    fn PumpOSInternal(&self) {
        // Preserve the source's reentrant-pump suppression while making the
        // same state safe for calls from concurrent Python workers.
        if self.mInsidePump.swap(true, Ordering::AcqRel) {
            return;
        }
        let _guard = PumpGuard(&self.mInsidePump);
        crate::BlueRuntime::shared().Pump();
    }
}
