use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use portable_pty::ChildKiller;
use tracing::warn;

use crate::pty_executor::PtyExecutorError;

pub fn with_timeout<'scope, R, F>(
    timeout: Option<Duration>,
    mut killer: Box<dyn ChildKiller + Send + Sync>,
    s: &'scope thread::Scope<'scope, '_>,
    f: F,
) -> Result<R, PtyExecutorError>
where
    F: FnOnce() -> R,
{
    if let Some(timeout) = timeout {
        let finished = Arc::new(AtomicBool::new(false));
        let finished_clone = finished.clone();

        let t = s.spawn(move || {
            thread::park_timeout(timeout);
            if !finished_clone.load(Ordering::SeqCst) {
                let _ = killer.kill();
                warn!("Command execution was terminated due to timeout");
            }
        });

        let result = f();
        finished.store(true, Ordering::SeqCst);

        t.thread().unpark();
        t.join()
            .map_err(|e| PtyExecutorError::ThreadJoinFailed(format!("{e:?}")))?;

        Ok(result)
    } else {
        Ok(f())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portable_pty::ChildKiller;
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::Duration,
    };

    #[derive(Debug)]
    struct FakeKiller {
        killed: Arc<AtomicBool>,
    }

    impl FakeKiller {
        fn new() -> Self {
            Self {
                killed: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    impl ChildKiller for FakeKiller {
        fn kill(&mut self) -> std::io::Result<()> {
            self.killed.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
            Box::new(Self {
                killed: self.killed.clone(),
            })
        }
    }

    #[test]
    fn test_with_timeout_no_timeout() {
        let killer = Box::new(FakeKiller::new());
        let result =
            thread::scope(|s| with_timeout(Some(Duration::from_millis(500)), killer, s, || 42))
                .unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_with_timeout_none() {
        let killer = Box::new(FakeKiller::new());
        let result = thread::scope(|s| with_timeout(None, killer, s, || 123)).unwrap();
        assert_eq!(result, 123);
    }

    #[test]
    fn test_with_timeout_triggers_kill() {
        let fake_killer = FakeKiller::new();
        let killed_flag = fake_killer.killed.clone();

        let killer: Box<dyn ChildKiller + Send + Sync> = Box::new(fake_killer);

        let start = std::time::Instant::now();
        let result = thread::scope(|s| {
            with_timeout(Some(Duration::from_millis(200)), killer, s, || {
                thread::sleep(Duration::from_millis(500));
                999
            })
        });

        assert!(
            killed_flag.load(Ordering::SeqCst),
            "Expected killer.kill() to be called"
        );
        assert!(
            result.is_ok(),
            "with_timeout should still return result of f()"
        );
        assert!(
            start.elapsed() < Duration::from_millis(600),
            "Timeout should cut execution"
        );
    }
}
