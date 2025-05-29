use algoparam::{AlgoParamSet};
use core::{ffi::c_void};
use std::{any::Any, slice};

pub mod algoparam;
pub mod util;
pub trait Algorithm : Send + Sync {
    // Returns an AlgoParamSet with basename as name. Each algorithm parameter uses self_ref for control.
    // Submodules must be instantiated as Rc<RefCell<>> and the corresponding parameters inserted in the tree by this method.
    fn init(&mut self, fs: i32, states: usize);
    // Returns the parameter set and the associated storage for using with the setter
    fn get_parameters(&self, basename: &str, displayname: &str) -> (AlgoParamSet, Box<dyn Any>);
    fn process(&self, parameter_zone: &Box<dyn Any>, outputs: &[&mut [f32]], inputs: &[&[f32]]);
    fn send_midi(&self, data: &[u8], timestamp: u64);
}


pub struct SoundModule {
    pub algo_state: Box<dyn Algorithm>,
    pub param: AlgoParamSet,
    pub parameter_zone: Box<dyn Any>,
}

impl SoundModule {
    pub fn new(mut algo: Box<dyn Algorithm>, fs: i32, states: usize) -> SoundModule {

        algo.init(fs, states);
        let params = algo.get_parameters("Root", "Root");
        SoundModule { algo_state: algo, param: params.0, parameter_zone: params.1 }
    }
}

fn as_soundmodule<'a>(this: *mut c_void) -> &'a mut SoundModule {
    let mut _mod = this as *mut SoundModule;
    unsafe { _mod.as_mut().unwrap() } 
}

#[unsafe(no_mangle)]
pub extern "C" fn soundmodule_release(this: *mut c_void) {
    if !this.is_null() {
        unsafe {
            drop(Box::from_raw(as_soundmodule(this) as *mut _));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn soundmodule_get_params(this: *mut c_void) -> *const c_void {
    let myself = as_soundmodule(this);
    let ptr = &myself.param as *const _ as *const c_void;
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn soundmodule_send_midi(this: *mut c_void, data: *const u8, len: usize, timestamp: u64) {
    let myself = as_soundmodule(this);
    let box_ref = &myself.algo_state;
    let data = unsafe { slice::from_raw_parts(data, len) };
    box_ref.send_midi(data,timestamp);
}

#[unsafe(no_mangle)]
pub extern "C" fn soundmodule_set_parameter(this: *mut c_void, address: u64, value: f32) {
    let myself = as_soundmodule(this);
    let _ = myself.param.set(value,address);
}

#[unsafe(no_mangle)]
pub extern "C" fn soundmodule_get_parameter(this: *mut c_void, address: u64) -> f32 {
    let myself = as_soundmodule(this);
    myself.param.get(address).unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn soundmodule_run(
    this: *mut c_void, 
    left_out: *mut f32,
    right_out: *mut f32,
    left_in: *const f32,
    right_in: *const f32,
    blksiz: u32) {
        let bz = blksiz as usize;
        
        let lo = unsafe {slice::from_raw_parts_mut(left_out, bz) };
        let ro = unsafe {slice::from_raw_parts_mut(right_out, bz) };
        let li = unsafe {slice::from_raw_parts(left_in, bz) };
        let ri = unsafe {slice::from_raw_parts(right_in, bz) };
        let myself = as_soundmodule(this);
        
        let output = [lo,ro];
        let input = [li,ri];

        myself.algo_state.process(&myself.parameter_zone, &output, &input);
    }