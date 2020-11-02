#![crate_type = "dylib"]

use std::ffi::CStr;
use std::mem;

use libc::{c_char, c_double, size_t};
use svg2polylines::{parse, CoordinatePair};

/// Structure that contains a pointer to the coordinate pairs as well as the
/// number of coordinate pairs. It is only used for C interop.
#[derive(Debug)]
#[repr(C)]
pub struct Polyline {
    ptr: *mut CoordinatePair,
    len: size_t,
}

#[no_mangle]
pub extern "C" fn svg_str_to_polylines(
    svg: *const c_char,
    tol: c_double,
    polylines: *mut *mut Polyline,
    polylines_len: *mut size_t,
) -> u8 {
    // Convert C string to Rust string
    let c_str = unsafe {
        assert!(!svg.is_null());
        CStr::from_ptr(svg)
    };
    let r_str = c_str.to_str().unwrap();

    // Process
    match parse(r_str, tol) {
        Ok(vec) => {
            // Convert `Vec<Vec<CoordinatePair>>` to `Vec<Polyline>`
            let mut tmp_vec: Vec<Polyline> = vec
                .into_iter()
                .map(|mut v| {
                    v.shrink_to_fit();
                    let p = Polyline {
                        ptr: v.as_mut_ptr(),
                        len: v.len(),
                    };
                    mem::forget(v);
                    p
                })
                .collect();
            tmp_vec.shrink_to_fit();
            assert!(tmp_vec.len() == tmp_vec.capacity());

            // Return number of polylines
            unsafe {
                *polylines_len = tmp_vec.len() as size_t;
            }

            // Return pointer to data
            unsafe {
                *polylines = tmp_vec.as_mut_ptr();
            }

            // Prevent memory from being deallocated
            mem::forget(tmp_vec);

            0
        }
        Err(_) => 1,
    }
}

#[no_mangle]
pub extern "C" fn free_polylines(polylines: *mut Polyline, polylines_len: size_t) {
    unsafe {
        for p in Vec::from_raw_parts(polylines, polylines_len as usize, polylines_len as usize) {
            Vec::from_raw_parts(p.ptr, p.len as usize, p.len as usize);
        }
    }
}
