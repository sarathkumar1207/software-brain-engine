use crate::EngineEvent;
use std::sync::{Arc, Mutex};

pub type EventHandler = Arc<dyn Fn(&EngineEvent) + Send + Sync + 'static>;

#[derive(Clone, Default)]
pub struct EventBus {
    handlers: Arc<Mutex<Vec<EventHandler>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, handler: EventHandler) {
        self.handlers.lock().expect("event bus lock").push(handler);
    }

    pub fn publish(&self, event: EngineEvent) {
        let handlers = self.handlers.lock().expect("event bus lock").clone();
        for handler in handlers {
            handler(&event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileEventKind;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn publishes_to_subscribers() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicUsize::new(0));
        let seen = count.clone();
        bus.subscribe(Arc::new(move |_| {
            seen.fetch_add(1, Ordering::SeqCst);
        }));

        bus.publish(EngineEvent::FileChanged {
            path: "src/app.ts".into(),
            kind: FileEventKind::Modified,
        });

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
