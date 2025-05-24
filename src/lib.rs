use algoparam::AlgoParamSet;
use core::{ffi::c_void};
use std::{slice};

pub trait Algorithm : Send + Sync {
    // Returns an AlgoParamSet with basename as name. Each algorithm parameter uses self_ref for control.
    // Submodules must be instantiated as Rc<RefCell<>> and the corresponding parameters inserted in the tree by this method.
    fn get_parameters(&self, basename: &str) -> AlgoParamSet;
    fn process(&self, outputs: &[&mut [f32]], inputs: &[&[f32]]);
}

pub struct SoundModule {
    pub algo: Box<dyn Algorithm>,
    pub param: AlgoParamSet,
}

fn as_soundmodule(this: *mut c_void) -> *mut SoundModule {
    this as *mut SoundModule
}

#[unsafe(no_mangle)]
pub extern "C" fn soundmodule_release(this: *mut c_void) {
    if !this.is_null() {
        unsafe {
            drop(Box::from_raw(as_soundmodule(this)));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn soundmodule_get_params(this: *mut c_void) -> *const c_void {
    let myself = unsafe { as_soundmodule(this).as_ref().unwrap() };
    let ptr = &myself.param as *const _ as *const c_void;
    ptr
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
        let myself = unsafe {as_soundmodule(this).as_ref().unwrap() };
        
        let output = [lo,ro];
        let input = [li,ri];

        myself.algo.process(&output, &input);
    }