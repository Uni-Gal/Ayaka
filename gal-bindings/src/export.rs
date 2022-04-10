use crate::*;

pub unsafe fn wit_bindgen_dispatch<T: Export>(arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> i32 {
    let len0 = arg1 as usize;
    let base2 = arg2;
    let len2 = arg3;
    let mut result2 = Vec::with_capacity(len2 as usize);
    for i in 0..len2 {
        let base = base2 + i * 16;
        result2.push(match i32::from(*((base + 0) as *const u8)) {
            0 => RawValue::Unit,
            1 => RawValue::Bool(match i32::from(*((base + 8) as *const u8)) {
                0 => false,
                1 => true,
                _ => panic!("invalid enum discriminant"),
            }),
            2 => RawValue::Num(*((base + 8) as *const i64)),
            3 => RawValue::Str({
                let len1 = *((base + 12) as *const i32) as usize;

                String::from_utf8(Vec::from_raw_parts(
                    *((base + 8) as *const i32) as *mut _,
                    len1,
                    len1,
                ))
                .unwrap()
            }),
            _ => panic!("invalid enum discriminant"),
        });
    }
    std::alloc::dealloc(
        base2 as *mut _,
        std::alloc::Layout::from_size_align_unchecked((len2 as usize) * 16, 8),
    );
    let result3 = T::dispatch(
        String::from_utf8(Vec::from_raw_parts(arg0 as *mut _, len0, len0)).unwrap(),
        result2,
    );
    let (result7_0, result7_1, result7_2, result7_3) = match result3 {
        None => (0i32, 0i32, 0i64, 0i32),
        Some(e) => {
            let (result6_0, result6_1, result6_2) = match e {
                RawValue::Unit => (0i32, 0i64, 0i32),
                RawValue::Bool(e) => {
                    let result4 = match e {
                        false => 0i32,
                        true => 1i32,
                    };

                    (1i32, i64::from(result4), 0i32)
                }
                RawValue::Num(e) => (2i32, wit_bindgen_rust::rt::as_i64(e), 0i32),
                RawValue::Str(e) => {
                    let vec5 = (e.into_bytes()).into_boxed_slice();
                    let ptr5 = vec5.as_ptr() as i32;
                    let len5 = vec5.len() as i32;
                    core::mem::forget(vec5);

                    (3i32, i64::from(ptr5), len5)
                }
            };

            (1i32, result6_0, result6_1, result6_2)
        }
    };
    let ptr8 = RET_AREA.as_mut_ptr() as i32;
    *((ptr8 + 24) as *mut i32) = result7_3;
    *((ptr8 + 16) as *mut i64) = result7_2;
    *((ptr8 + 8) as *mut i32) = result7_1;
    *((ptr8 + 0) as *mut i32) = result7_0;
    ptr8
}
pub trait Export {
    fn dispatch(name: String, args: Vec<RawValue>) -> Option<RawValue>;
}
static mut RET_AREA: [i64; 4] = [0; 4];

#[macro_export]
macro_rules! decl_dispatch {
    ($t:ty) => {
        #[no_mangle]
        unsafe extern "C" fn dispatch(arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> i32 {
            $crate::wit_bindgen_dispatch::<$t>(arg0, arg1, arg2, arg3)
        }
    };
}
