use std::{
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant}
};

const TIMER_FREQ: f64 = 1.0 / 60.0; // 60Hz

pub struct TimerClock {
    delay: Arc<AtomicU8>,
    sound: Arc<AtomicU8>,
    stop_flag: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl TimerClock {
    pub fn new(delay: Arc<AtomicU8>, sound: Arc<AtomicU8>) -> Self {
        Self {
            delay,
            sound,
            stop_flag: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }


    pub fn start(&mut self) { // todo: should I add some Stopped/Running states for CPU/TimerClock?
        let thread_stop = self.stop_flag.clone();
        let delay_clone = self.delay.clone();
        let sound_clone = self.sound.clone();

        let handle = thread::spawn(move || {
            let tick = Duration::from_secs_f64(TIMER_FREQ);
            let mut next = Instant::now() + tick;
            while !thread_stop.load(Ordering::Relaxed) {
                let now = Instant::now();
                if now >= next {
                    if delay_clone.load(Ordering::Relaxed) > 0 {
                        delay_clone.fetch_sub(1, Ordering::Relaxed);
                    }
                    if sound_clone.load(Ordering::Relaxed) > 0 {
                        sound_clone.fetch_sub(1, Ordering::Relaxed);
                    }
                    next += tick;
                } else {
                    thread::sleep(next - now);
                }
            }
        });

        self.handle = Some(handle);
    }

    pub fn shutdown(&mut self) { // todo: should be called only on started
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for TimerClock {
    fn drop(&mut self) {
        self.shutdown();
    }
}