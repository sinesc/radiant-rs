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

#[allow(unused_variables)]
pub fn mainloop<F>(interval: Duration, mut callback: F) where F: FnMut(f32) -> bool {

    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();

    let second = Duration::new(1, 0);
    let mut second_elapsed = Duration::new(0, 0);
    let mut frames_elapsed = 0;

    loop {

        let now = Instant::now();
        let delta = now - previous_clock;
        let delta_f32 = delta.as_secs() as f32 + (delta.subsec_nanos() as f64 / 1000000000.0) as f32;

        if callback(delta_f32) == false {
            break;
        }

        // determine thread sleep to maintain X FPS
        accumulator += delta;

        while accumulator >= interval {
            accumulator -= interval;
            // if you have a game, update the state here
        }

        // framerate print
        second_elapsed += now - previous_clock;
        frames_elapsed += 1;

        if second_elapsed >= second {
            //println!("Frames rendered: {}", frames_elapsed);
            second_elapsed -= second;
            frames_elapsed = 0;
        }

        previous_clock = now;
        thread::sleep(interval - accumulator);
    }
}
