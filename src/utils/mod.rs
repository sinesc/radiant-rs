use std::time::{Instant, Duration};
use std::thread;
use num::traits::*;

pub use avec::AVec;

pub struct Rng (f64);

impl Rng {

    pub fn new<T>(seed: T) -> Rng where T: FromPrimitive + ToPrimitive + Copy {
        Rng(seed.to_f64().unwrap())
    }

    pub fn get<T>(self: &mut Self) -> T where T: FromPrimitive + ToPrimitive + Copy {

        let large = self.0.sin() * 100000000.0;
        self.0 += 1.0;

        T::from_f64(large - large.floor()).unwrap()
    }

    pub fn range<T>(self: &mut Self, min: T, max: T) -> T where T: FromPrimitive + ToPrimitive + Copy {

        let large = self.0.sin() * 100000000.0;
        self.0 += 1.0;

        let base = (large - large.floor()) as f64;
        let min = min.to_f64().unwrap();
        let max = max.to_f64().unwrap();
        T::from_f64(min + base * (max - min)).unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct MainloopState {
    pub delta       : Duration,
    pub delta_f32   : f32,
    pub elapsed     : Duration,
    pub elapsed_f32 : f32,
    pub fps         : u32,
    pub state_id    : u32,
}

#[allow(unused_variables)]
pub fn mainloop<F, G>(interval: Duration, mut state_callback: F, mut render_callback: G) where F: FnMut(MainloopState) -> bool, G: FnMut(MainloopState) -> bool {

    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();
    let start_clock = previous_clock;

    let second = Duration::new(1, 0);
    let mut second_elapsed = Duration::new(0, 0);
    let mut frames_elapsed = 0;
    let mut fps = 0;

    loop {

        let now = Instant::now();
        let delta = now - previous_clock;
        let elapsed = now - start_clock;

        let mut state_info = MainloopState {
            delta       : delta,
            delta_f32   : delta.as_secs() as f32 + (delta.subsec_nanos() as f64 / 1000000000.0) as f32,
            elapsed     : elapsed,
            elapsed_f32 : elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f64 / 1000000000.0) as f32,
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

        if second_elapsed >= second {
            fps = frames_elapsed;
            second_elapsed -= second;
            frames_elapsed = 0;
        }

        previous_clock = now;
        thread::sleep(interval - accumulator);
    }
}

#[allow(unused_variables)]
pub fn renderloop<G>(mut render_callback: G) where G: FnMut(MainloopState) -> bool {

    let mut previous_clock = Instant::now();
    let start_clock = previous_clock;

    let second = Duration::new(1, 0);
    let mut second_elapsed = Duration::new(0, 0);
    let mut frames_elapsed = 0;
    let mut fps = 0;

    loop {

        let now = Instant::now();
        let delta = now - previous_clock;
        let elapsed = now - start_clock;

        let mut state_info = MainloopState {
            delta       : delta,
            delta_f32   : delta.as_secs() as f32 + (delta.subsec_nanos() as f64 / 1000000000.0) as f32,
            elapsed     : elapsed,
            elapsed_f32 : elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f64 / 1000000000.0) as f32,
            fps         : fps,
            state_id    : 0,
        };

        if render_callback(state_info) == false {
            break;
        }

        // framerate print
        second_elapsed += now - previous_clock;
        frames_elapsed += 1;

        if second_elapsed >= second {
            fps = frames_elapsed;
            second_elapsed -= second;
            frames_elapsed = 0;
        }

        previous_clock = now;
    }
}
