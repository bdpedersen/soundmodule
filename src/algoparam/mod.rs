use std::{ffi::{c_char, c_void, CString}, ptr::null};


#[derive(Debug)]
pub struct OutOfRangeError;

impl std::fmt::Display for OutOfRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Value is out of range")
    }
}

impl std::error::Error for OutOfRangeError {}



pub const KEY_NOT_FOUND: u64 = 0xffff_ffff_ffff_ffff;
pub const KEY_MASK: u64  = 0x00ff_0000_0000_0000;

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum AlgoParamUnit {
    GENERIC,
    INDEXED,
    BOOLEAN,
    PERCENT,
    SECONDS,
    SAMPLES,
    HERTZ,
    CENTS,
    SEMITONES,
    MIDINOTENUMBER,
    DECIBELS,
    LINEARGAIN,
    DEGREES,
    EQUALPOWERCROSSFADE,
    MILLISECONDS,
}



pub enum AlgoParamNode {
    Param(AlgoParam),
    ParamSet(AlgoParamSet),
}

pub struct AlgoParam {
    pub identifier: CString,
    pub name: CString,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: AlgoParamUnit,
    pub setter: Box<dyn Fn(f32)->()>,
    pub getter: Box<dyn Fn()->f32>,
    pub dependents: Vec<CString>,                         // logical names
    pub dependents_ptr: Option<Box<[*const c_char]>>,     // raw array for FFI - content owned by the logical names above.
}

impl AlgoParam {
    pub fn new(key: &str, name: &str, min: f32, max: f32, default:  f32, unit: AlgoParamUnit, 
                setter: Box<dyn Fn(f32)->()>, getter: Box<dyn Fn()->f32>, dependents: &[&str]) -> AlgoParam{
        let _key = CString::new(key).expect("Should not fail ...");
        let _name = CString::new(name).expect("Should not fail ...");
        let _dependents: Vec<CString> = dependents.iter().map(|s| CString::new(*s).expect("null byte in dependent name")).collect();
        let _dependents_ptr = if _dependents.is_empty() {
            None
        } else {
            let mut raw_ptrs: Vec<*const c_char> = _dependents.iter().map(|s| s.as_ptr()).collect();
            raw_ptrs.push(null());
            Some(raw_ptrs.into_boxed_slice())
        };

        AlgoParam {
            identifier: _key, 
            name: _name, 
            min, 
            max, 
            default,
            unit, 
            setter, 
            getter, 
            dependents: _dependents, 
            dependents_ptr: _dependents_ptr
        }
    }

    pub fn dependents_as_raw(&self) -> *const *const c_char {
        self.dependents_ptr.as_ref().map(|slice| slice.as_ptr()).unwrap_or(null())
    }
}

pub struct AlgoParamSet {
    pub identifier: CString,
    pub name: CString,
    pub children: Vec<AlgoParamNode>,
}

impl AlgoParamSet {
    pub fn new(identifier: &str, name: &str) -> AlgoParamSet {
        let children = Vec::<AlgoParamNode>::new();
        let _identifier = CString::new(identifier).expect("Should not have failed here");
        let _name = CString::new(name).expect("Should not have failed here...");
        AlgoParamSet { identifier: _identifier, name: _name, children }
    }

    pub fn add(&mut self, child: AlgoParamNode) -> Result<(),OutOfRangeError> {
        if self.children.len() == 254 {
            return Err(OutOfRangeError)
        }
        self.children.push(child);
        Ok(())
    }

    pub fn get_param_mut(&mut self, key: u64) -> Option<&mut AlgoParam> {
        let idx = key >> 56;
        if idx >= self.children.len() as u64 {
            None
        } else {
            match &mut self.children[idx as usize] {
                AlgoParamNode::Param(param) => {
                    Some(param) 
                } ,
                AlgoParamNode::ParamSet(set) => {
                    set.get_param_mut(key<<8)
                },
            }
        }
    }

    pub fn find_first_set(&self, basekey: u64) -> Option<(&AlgoParamSet,u64)> {
        if basekey == KEY_NOT_FOUND {
            // Just find the first one and return that if it exists
            let mut idx = 0;
            while idx < self.children.len() {
                if let AlgoParamNode::ParamSet(v) = &self.children[idx] {
                    return Some((v,(idx as u64) << 56 | 0x00ff_ffff_ffff_ffff));
                }
                idx += 1;
            }
            return None;
        } else {
            let mut head = basekey >> 56;
            let tail = basekey << 8 | 0xffu64;
            if let AlgoParamNode::ParamSet(set) = &self.children[head as usize] {
                let (v,newbase) = set.find_first_set(tail)?;
                head <<= 56;
                head |= newbase >> 8;
                Some((v,head))
            } else {
                None
            }

        }
    }

    pub fn find_next_set(&self, basekey: u64) -> Option<(&AlgoParamSet,u64)> {
        if basekey & KEY_MASK == KEY_MASK {
            // We should iterate in this one
            let mut idx =  1 + (basekey as usize >> 56) ;
            while idx < self.children.len() {
                if let AlgoParamNode::ParamSet(v) = &self.children[idx] {
                    return Some((v,(idx as u64) << 56 | 0x00ff_ffff_ffff_ffff));
                }
                idx += 1;
            }
            return None;
        } else {
            // We need to go one level down
            let mut head = basekey >> 56;
            let tail = basekey << 8 | 0xffu64;
            if let AlgoParamNode::ParamSet(set) = &self.children[head as usize] {
                let (v,newbase) = set.find_next_set(tail)?;
                head <<= 56;
                head |= newbase >> 8;
                Some((v,head))
            } else {
                None
            }
        }
    }

    pub fn find_first_param(&self, basekey: u64) -> Option<(&AlgoParam,u64)> {
        if basekey == KEY_NOT_FOUND {
            // Just find the first one and return that if it exists
            let mut idx = 0;
            while idx < self.children.len() {
                if let AlgoParamNode::Param(v) = &self.children[idx] {
                    return Some((v,(idx as u64) << 56 | 0x00ff_ffff_ffff_ffff));
                }
                idx += 1;
            }
            return None;
        } else {
            let mut head = basekey >> 56;
            let tail = (basekey << 8) | 0xffu64;
            if let AlgoParamNode::ParamSet(set) = &self.children[head as usize] {
                let (v,newbase) = set.find_first_param(tail)?;
                head <<= 56;
                head |= newbase >> 8;
                Some((v,head))
            } else {
                None
            }
        }
    }

    pub fn find_next_param(&self, basekey: u64) -> Option<(&AlgoParam,u64)> {
        if basekey & KEY_MASK == KEY_MASK {
            // Just find the next first one and return that if it exists
            let mut idx =  1 + (basekey as usize >> 56) ;
            while idx < self.children.len() {
                if let AlgoParamNode::Param(v) = &self.children[idx] {
                    return Some((v,(idx as u64) << 56 | 0x00ff_ffff_ffff_ffff));
                }
                idx += 1;
            }
            return None;
        } else {
            let mut head = basekey >> 56;
            let tail = basekey << 8 | 0xffu64;
            if let AlgoParamNode::ParamSet(set) = &self.children[head as usize] {
                let (v,newbase) = set.find_next_param(tail)?;
                head <<= 56;
                head |= newbase >> 8;
                Some((v,head))
            } else {
                None
            }
        }
    }

    pub fn set(&mut self, value: f32, key: u64) -> Result<(),OutOfRangeError> {
        if let Some(param) = self.get_param_mut(key) {
            let val = (param.setter)(value);
            return Ok(val);
        } 
        Err(OutOfRangeError)
    }

    pub fn get(&mut self, key: u64) -> Result<f32, OutOfRangeError> {
        if let Some(param) = self.get_param_mut(key) {
            let val = (param.getter)();
            return Ok(val);
        } 
        Err(OutOfRangeError)
    }
}


/// C interface. Note that the API does not offer any way to actually get a tree - that needs to be supplied by
/// a client module. It is expected that tree is of the type *const AlgoParamSet

#[repr(C)]
pub struct AlgoCParam {
    pub key: *const c_char,
    pub name: *const c_char,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub dtype: i32,
    pub dependents: *const *const c_char,
}

impl AlgoCParam {
    fn null() -> AlgoCParam {
        AlgoCParam {
            key: null(),
            name: null(),
            min: 0.0,
            max: 0.0,
            default: 0.0,
            dtype: 0,
            dependents: null()
        }
    }

    fn new(from: &AlgoParam) -> AlgoCParam{
        AlgoCParam {
            key: from.identifier.as_ptr(),
            name: from.name.as_ptr(),
            min: from.min,
            max: from.max,
            default: from.default,
            dtype: from.unit as i32,
            dependents: from.dependents_as_raw(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn algoparam_get_first_set(tree: *const c_void, basekey: *mut u64) -> *const c_char {
    let set = unsafe { &*(tree as *const AlgoParamSet)};
    let bkey = unsafe { *basekey };
    if let Some(newkey) = set.find_first_set(bkey) {
        unsafe { *basekey = newkey.1 };
        return newkey.0.name.as_ptr();
    } else {
        unsafe { *basekey = KEY_NOT_FOUND };
        return null();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn algoparam_get_next_set(tree: *const c_void, basekey: *mut u64) -> *const c_char {
    let set = unsafe { &*(tree as *const AlgoParamSet)};
    let bkey = unsafe { *basekey };
    if let Some(newkey) = set.find_next_set(bkey) {
        unsafe { *basekey = newkey.1 };
        return newkey.0.name.as_ptr();
    } else {
        unsafe { *basekey = KEY_NOT_FOUND };
        return null();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn algoparam_get_first_param(tree: *const c_void, basekey: *mut u64) -> AlgoCParam {
    let set = unsafe { &*(tree as *const AlgoParamSet)};
    let bkey = unsafe { *basekey };
    if let Some(newkey) = set.find_first_param(bkey) {
        unsafe { *basekey = newkey.1 };
        return AlgoCParam::new(newkey.0);
    } else {
        unsafe { *basekey = KEY_NOT_FOUND };
        return AlgoCParam::null();
    }

}

#[unsafe(no_mangle)]
pub extern "C" fn algoparam_get_next_param(tree: *const c_void, basekey: *mut u64) -> AlgoCParam {
    let set = unsafe { &*(tree as *const AlgoParamSet)};
    let bkey = unsafe { *basekey };
    if let Some(newkey) = set.find_next_param(bkey) {
        unsafe { *basekey = newkey.1 };
        return AlgoCParam::new(newkey.0);
    } else {
        unsafe { *basekey = KEY_NOT_FOUND };
        return AlgoCParam::null();
    }

}


#[cfg(test)]
mod tests {
    use super::*;




    fn build_tree() -> AlgoParamSet {
        let mut root = AlgoParamSet::new("root", "Root");
        let mut subset1 = AlgoParamSet::new("subset1", "subset 1");
        let mut subset2 = AlgoParamSet::new("subset2", "subset 2");
        
        let param1_1 = AlgoParam::new("param1_1", "Param 1.1", 0.0, 1.0, 0.0, AlgoParamUnit::GENERIC,  Box::new(|_| {}), Box::new(|| 0.0), &[]);
        let param1_2 = AlgoParam::new("param1_2", "Param 1.2", 0.0, 1.0, 0.0, AlgoParamUnit::GENERIC,  Box::new(|_| {}), Box::new(|| 0.0), &[]);
        
        let _ = subset1.add(AlgoParamNode::Param(param1_1));
        let _ = subset1.add(AlgoParamNode::Param(param1_2));

        let param2_1 = AlgoParam::new("param2_1", "Param 2.1", 0.0, 1.0, 0.0, AlgoParamUnit::GENERIC,  Box::new(|_| {}), Box::new(|| 0.0), &[]);
        let param2_2 = AlgoParam::new("param2_2", "Param 2.2", 0.0, 1.0, 0.0, AlgoParamUnit::GENERIC,  Box::new(|_| {}), Box::new(|| 0.0), &[]);
        
        let _ = subset2.add(AlgoParamNode::Param(param2_1));
        let _ = subset2.add(AlgoParamNode::Param(param2_2));
        let _ = subset2.add(AlgoParamNode::ParamSet(AlgoParamSet::new("mock", "Mock Algorithm")));

        let param3 = AlgoParam::new("param3", "Param 3", 0.0, 1.0, 0.0, AlgoParamUnit::GENERIC,  Box::new(|_| {}), Box::new(|| 0.0), &[]);
        let _ = root.add(AlgoParamNode::ParamSet(subset1));
        let _ = root.add(AlgoParamNode::ParamSet(subset2));

        let _ = root.add(AlgoParamNode::Param(param3));

        root
    }

    fn as_strref(s: *const i8) -> &'static str {
        unsafe { std::ffi::CStr::from_ptr(s).to_str().unwrap() }
    }

    fn as_voidptr<T>(s: &T) -> *const c_void {
        s as *const _ as *const c_void
    }

    #[test]
    fn test_find_first_param() {
        let tree = build_tree();
        let mut basekey = KEY_NOT_FOUND;
        // Test first subset retrieval
        let first_subset = algoparam_get_first_set(as_voidptr(&tree), &mut basekey);
        assert_eq!(as_strref(first_subset), "subset1");
        let expected = (0x00u64 << 56) | 0x00ff_ffff_ffff_ffff;
        assert_eq!(basekey, expected);
        let subset1 = basekey;

        // Test next subset retrieval   
        let next_subset = algoparam_get_next_set(as_voidptr(&tree), &mut basekey);
        assert_eq!(as_strref(next_subset), "subset2");
        let expected = (0x01u64 << 56) | 0x00ff_ffff_ffff_ffff;
        assert_eq!(basekey, expected);
        let subset2 = basekey;
        let _ = algoparam_get_next_set(as_voidptr(&tree), &mut basekey);
        assert_eq!(basekey, KEY_NOT_FOUND);

        // Test first parameter retrieval
        basekey = KEY_NOT_FOUND;
        let first_param = algoparam_get_first_param(as_voidptr(&tree), &mut basekey);
        assert_eq!(as_strref(first_param.key), "param3");
        let expected = (0x02u64 << 56) | 0x00ff_ffff_ffff_ffff;
        assert_eq!(basekey, expected);

        // Test next parameter retrieval - this should fail
        let _ = algoparam_get_next_param(as_voidptr(&tree), &mut basekey);
        assert_eq!(basekey, KEY_NOT_FOUND);

        // Test first parameter retrieval from subset1
        basekey = subset1;
        let first_param_subset1 = algoparam_get_first_param(as_voidptr(&tree), &mut basekey);
        assert_eq!(as_strref(first_param_subset1.key), "param1_1");
        let expected = (0x0000 << 48) | 0x0000_ffff_ffff_ffff;
        assert_eq!(basekey, expected);

        // Test next parameter retrieval from subset1
        let next_param_subset1 = algoparam_get_next_param(as_voidptr(&tree), &mut basekey);
        assert_eq!(as_strref(next_param_subset1.key), "param1_2");
        let expected = (0x0001u64 << 48) | 0x0000_ffff_ffff_ffff;
        assert_eq!(basekey, expected);

        // next retrieval should fail
        let _ = algoparam_get_next_param(as_voidptr(&tree), &mut basekey);
        assert_eq!(basekey, KEY_NOT_FOUND);

        // Check that we can't find any sets
        basekey = subset1;
        let _= algoparam_get_first_set(as_voidptr(&tree), &mut basekey);
        assert_eq!(basekey, KEY_NOT_FOUND);


        // repeat for subset2
        basekey = subset2;
        let first_param_subset2 = algoparam_get_first_param(as_voidptr(&tree), &mut basekey);
        assert_eq!(as_strref(first_param_subset2.key), "param2_1");
        let expected = (0x0100u64 << 48) | 0x0000_ffff_ffff_ffff;
        assert_eq!(basekey, expected);

        // Test next parameter retrieval from subset2
        let next_param_subset2 = algoparam_get_next_param(as_voidptr(&tree), &mut basekey);
        assert_eq!(as_strref(next_param_subset2.key), "param2_2");
        let expected = (0x0101u64 << 48) | 0x0000_ffff_ffff_ffff;

        assert_eq!(basekey, expected);

        // next retrieval should fail
        let _ = algoparam_get_next_param(as_voidptr(&tree), &mut basekey);
        assert_eq!(basekey, KEY_NOT_FOUND); 

        basekey = subset2;
        let subsubset1= algoparam_get_first_set(as_voidptr(&tree), &mut basekey);
        assert_eq!(as_strref(subsubset1),"mock");

        let _= algoparam_get_next_set(as_voidptr(&tree), &mut basekey);
        assert_eq!(basekey, KEY_NOT_FOUND);


    }
        

}