use std::time::{Instant, Duration};
use std::thread;

/// Passed to `renderloop()` and `mainloop()` callbacks.
#[derive(Copy, Clone)]
pub struct LoopState {
    /// Time between frames in as Duration.
    pub delta       : Duration,
    /// Time between frames in seconds.
    pub delta_f32   : f32,
    /// Time since loop was started as Duration.
    pub elapsed     : Duration,
    /// Time since loop was started in seconds.
    pub elapsed_f32 : f32,
    /// Current frame id.
    pub frame_id    : u64,
    /// Current framerate.
    pub fps         : u32,
    /// Current state id for `mainloop()`'s `state_callback`.
    pub state_id    : u32,
}

/// Main loop helper function. Provides callbacks for game state changes and rendering. Calls
/// state_callback multiple times if the actual render_callback call interval exceeds the given interval.
/// Both callbacks receive a LoopState object containing frame delta and fps data.
pub fn mainloop<F, G>(interval: Duration, mut state_callback: F, mut render_callback: G) where F: FnMut(LoopState) -> bool, G: FnMut(LoopState) -> bool {

    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();
    let start_clock = previous_clock;

    let second = Duration::new(1, 0);
    let mut second_elapsed = Duration::new(0, 0);
    let mut frames_elapsed = 0;
    let mut fps = 0;
    let mut frame_id = 0;

    loop {

        let now = Instant::now();
        let delta = now - previous_clock;
        let elapsed = now - start_clock;

        let mut state_info = LoopState {
            delta       : delta,
            delta_f32   : delta.as_secs() as f32 + (delta.subsec_nanos() as f64 / 1000000000.0) as f32,
            elapsed     : elapsed,
            elapsed_f32 : elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f64 / 1000000000.0) as f32,
            frame_id    : frame_id,
            fps         : fps,
            state_id    : 0,
        };

        if render_callback(state_info) == false {
            break;
        }

        // determine thread sleep to maintain X FPS
        accumulator += delta;

        while accumulator >= interval {
            accumulator -= interval;
            if state_callback(state_info) == false {
                break;
            }
            state_info.state_id += 1;
        }

        // framerate print
        second_elapsed += now - previous_clock;
        frames_elapsed += 1;
        frame_id += 1;

        if second_elapsed >= second {
            fps = frames_elapsed;
            second_elapsed -= second;
            frames_elapsed = 0;
        }

        previous_clock = now;
        thread::sleep(interval - accumulator);
    }
}

/// Render loop helper function. Provides given callback with frame delta and fps data.
/// This is a more simple alternative to the main loop helper function.
pub fn renderloop<G>(mut render_callback: G) where G: FnMut(LoopState) -> bool {

    let mut previous_clock = Instant::now();
    let start_clock = previous_clock;

    let second = Duration::new(1, 0);
    let mut second_elapsed = Duration::new(0, 0);
    let mut frames_elapsed = 0;
    let mut fps = 0;
    let mut frame_id = 0;

    loop {

        let now = Instant::now();
        let delta = now - previous_clock;
        let elapsed = now - start_clock;

        let state_info = LoopState {
            delta       : delta,
            delta_f32   : delta.as_secs() as f32 + (delta.subsec_nanos() as f64 / 1000000000.0) as f32,
            elapsed     : elapsed,
            elapsed_f32 : elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f64 / 1000000000.0) as f32,
            frame_id    : frame_id,
            fps         : fps,
            state_id    : 0,
        };

        if render_callback(state_info) == false {
            break;
        }

        // framerate print
        second_elapsed += now - previous_clock;
        frames_elapsed += 1;
        frame_id += 1;

        if second_elapsed >= second {
            fps = frames_elapsed;
            second_elapsed -= second;
            frames_elapsed = 0;
        }

        previous_clock = now;
    }
}
