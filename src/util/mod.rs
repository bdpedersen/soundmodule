use std::sync::atomic::{AtomicU32, Ordering};



#[derive(Debug)]
pub struct AtomicF32 {
    inner: AtomicU32,
}

impl AtomicF32 {
    pub fn new(val: f32) -> Self {
        Self {
            inner: AtomicU32::new(val.to_bits()),
        }
    }

    pub fn load(&self, order: Ordering) -> f32 {
        f32::from_bits(self.inner.load(order))
    }

    pub fn store(&self, val: f32, order: Ordering) {
        self.inner.store(val.to_bits(), order)
    }

    pub fn swap(&self, val: f32, order: Ordering) -> f32 {
        f32::from_bits(self.inner.swap(val.to_bits(), order))
    }

    pub fn compare_exchange(
        &self,
        current: f32,
        new: f32,
        success: Ordering,
        failure: Ordering,
    ) -> Result<f32, f32> {
        self.inner
            .compare_exchange(
                current.to_bits(),
                new.to_bits(),
                success,
                failure,
            )
            .map(f32::from_bits)
            .map_err(f32::from_bits)
    }

    pub fn compare_exchange_weak(
        &self,
        current: f32,
        new: f32,
        success: Ordering,
        failure: Ordering,
    ) -> Result<f32, f32> {
        self.inner
            .compare_exchange_weak(
                current.to_bits(),
                new.to_bits(),
                success,
                failure,
            )
            .map(f32::from_bits)
            .map_err(f32::from_bits)
    }

    pub fn fetch_update<F>(
        &self,
        set_order: Ordering,
        fetch_order: Ordering,
        mut f: F,
    ) -> Result<f32, f32>
    where
        F: FnMut(f32) -> Option<f32>,
    {
        self.inner
            .fetch_update(set_order, fetch_order, |bits| {
                f(f32::from_bits(bits)).map(f32::to_bits)
            })
            .map(f32::from_bits)
            .map_err(f32::from_bits)
    }
}


pub struct Smooth {
    state: AtomicF32,
    target: AtomicF32,
    alpha: f32,
}

fn smooth_update(s: f32, t:f32, a: f32) -> f32 {
    let diff = s-t;
    if diff.abs() < 1e5 {
        t
    } else {
        t+a*(s-t)
    }
}

impl Smooth {
    pub fn new_with_value_ex(initial:f32, alpha: f32) -> Smooth {
        Smooth { 
            state: AtomicF32::new(initial), 
            target: AtomicF32::new(initial),
            alpha
        }

    }

    pub fn new_with_value(initial: f32) -> Smooth {
        Smooth::new_with_value_ex(initial,0.05)
    }

    pub fn new() -> Smooth {
        Smooth::new_with_value(0.0)
    }

    pub fn next(&self) -> f32 {
        self.state.fetch_update(Ordering::Relaxed, Ordering::Relaxed, 
            |x| Some(
                smooth_update(x, self.target.load(Ordering::Relaxed), self.alpha)
            ))
            .expect("update failed")
    }

    pub fn set(&self, target: f32) {
        self.target.store(target, Ordering::Relaxed);
    }

    pub fn next_targeting(&self, target :f32) -> f32 {
        self.set(target);
        self.next()
    }

    pub fn next_many(&self, output: &mut [f32]){
        let mut st = self.state.load(Ordering::Relaxed);
        let tgt = self.target.load(Ordering::Relaxed);
        for v in output.iter_mut() {
            st = smooth_update(st, tgt, self.alpha);
            *v = st
        }
        self.state.store(st, Ordering::Relaxed);
    }

}
