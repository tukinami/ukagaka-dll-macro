//! 伺かのDLL用マクロ。
//!
//! 伺かのDLLに使われる`load`、`request`、`unload`と、DLLのエントリポイントである`DllMain`を定義するマクロ集です。
//! おまけで、Dllへのパスを返す関数 [`read_dll_path_string`] も定義しています。
//!
//! マクロを使用するための型や関数を定義してあるので、使用するときは、
//!
//! ```rust
//! use ukagaka_dll_macro::*;
//! ```
//!
//! とグロブで`use`してください。
//!
//! # Examples
//!
//! ```
//! // lib.rs
//! use ukagaka_dll_macro::*;
//!
//! fn ukagaka_load() -> bool {
//!     if let Some(_dll_path) = read_dll_path_string() {
//!         // process with dll path
//!     }
//!     true
//! }
//!
//! fn ukagaka_request(_s: &[u8]) -> Vec<i8> {
//!     if let Some(_dll_path) = read_dll_path_string() {
//!         // process with dll path & s(contents of request).
//!     }
//!     // build response
//!     b"SAORI/1.0 200 OK\r\nResult:1\r\nCharset:UTF-8\r\n\r\n\0"
//!         .iter()
//!         .map(|v| *v as i8)
//!         .collect()
//! }
//!
//! define_dll_main!();
//! define_load!(ukagaka_load);
//! define_request!(ukagaka_request);
//! define_unload!();
//! ```
//!
//! [`read_dll_path_string`]: crate::read_dll_path_string

use std::sync::OnceLock;

use winapi::um::winbase::{GlobalAlloc, GMEM_FIXED};
use winapi::{shared::minwindef::MAX_PATH, um::libloaderapi::GetModuleFileNameW};

pub use std::ffi::c_long;
pub use winapi::{
    shared::minwindef::{BOOL, DWORD, FALSE, HGLOBAL, HINSTANCE, LPVOID, TRUE},
    um::{
        winbase::GlobalFree,
        winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH},
    },
};

static DLL_PATH: OnceLock<String> = OnceLock::new();

/// DLLへのパスを記録する関数。
///
/// この関数を`DLLMain`の`PROCESS_ATTACH`時に呼ぶと、それ以降 [`read_dll_path_string`] が`None`でなくなります。
///
/// # Safety
/// この関数は、内部で [`GetModuleFileNameW`] を使用しています。
///
/// [`read_dll_path_string`]: crate::read_dll_path_string
/// [`GetModuleFileNameW`]: winapi::um::libloaderapi::GetModuleFileNameW
pub unsafe fn register_dll_path_string(h_module: HINSTANCE) {
    let mut buf: [u16; MAX_PATH + 1] = [0; MAX_PATH + 1];
    unsafe {
        GetModuleFileNameW(h_module, buf.as_mut_ptr(), MAX_PATH as u32);
    }

    let p = buf.partition_point(|v| *v != 0);

    let _ = DLL_PATH.set(String::from_utf16_lossy(&buf[..p]));
}

/// DLLへのパスを返す関数。
///
/// [`register_dll_path_string`] が呼ばれていないと、`None`しか返しません。
///
/// [`register_dll_path_string`]: crate::register_dll_path_string
pub fn read_dll_path_string() -> Option<String> {
    DLL_PATH.get().cloned()
}

/// `i8`のスライスから、`HGLOBAL`を返す関数。
///
/// # Safety
/// この関数は、内部で [`GlobalAlloc`] 、[`from_raw_parts_mut`] を使用しています。
///
/// [`GlobalAlloc`]: winapi::um::winbase::GlobalAlloc
/// [`from_raw_parts_mut`]: std::slice::from_raw_parts_mut
pub unsafe fn slice_i8_to_hglobal(h_len: *mut c_long, data: &[i8]) -> HGLOBAL {
    let data_len = data.len();

    let h = unsafe { GlobalAlloc(GMEM_FIXED, data_len) };

    unsafe { *h_len = data_len as c_long };

    let h_slice = unsafe { std::slice::from_raw_parts_mut(h as *mut i8, data_len) };

    h_slice.copy_from_slice(&data[..data_len]);

    h
}

/// `HGLOBAL`から`Vec<u8>`を返す関数。
///
/// # Safety
/// この関数は内部で [`from_raw_parts`] を使用しています。
/// `len`で表わされる長さを妥当なものにしてください。
///
/// [`from_raw_parts`]: std::slice::from_raw_parts
pub fn hglobal_to_vec_u8(h: HGLOBAL, len: c_long) -> Vec<u8> {
    let mut vec = unsafe { std::slice::from_raw_parts(h as *const u8, len as usize).to_vec() };
    vec.push(0);

    vec
}

/// 関数`DLLMain`を定義するマクロ。
///
/// 引数は順番に、`DLL_PROCESS_ATTACH`時、`DLL_PROCESS_DETACH`時、`DLL_THREAD_ATTACH`時、`DLL_THREAD_DETACH`時の処理になります。
/// それぞれ省略可で、もし、途中を飛ばしたい場合、`()`を指定してください。それでその時点での処理はなくなります。
/// 引数なしなら、以下の動作のみになります。
/// 内部で`DLL_PROCESS_ATTACH`時に [`register_dll_path_string`] を呼んで、DLLへのパスを記録しています。
///
/// [`register_dll_path_string`]: crate::register_dll_path_string
#[macro_export]
macro_rules! define_dll_main {
    () => {
        define_dll_main!((), (), (), ());
    };
    ($process_attach:expr) => {
        define_dll_main!($process_attach, (), (), ());
    };

    ($process_attach:expr, $process_detach:expr) => {
        define_dll_main!($process_attach, $process_detach, (), ());
    };

    ($process_attach:expr, $process_detach:expr, $thread_attach:expr) => {
        define_dll_main!($process_attach, $process_detach, $thread_attach, ());
    };

    ($process_attach:expr, $process_detach:expr, $thread_attach:expr, $thread_detach:expr) => {
        #[no_mangle]
        pub unsafe extern "system" fn DllMain(
            h_module: HINSTANCE,
            ul_reason_for_call: DWORD,
            _l_reserved: LPVOID,
        ) -> BOOL {
            match ul_reason_for_call {
                DLL_PROCESS_ATTACH => {
                    unsafe {
                        register_dll_path_string(h_module);
                    }
                    $process_attach;
                }
                DLL_PROCESS_DETACH => {
                    $process_detach;
                }
                DLL_THREAD_ATTACH => {
                    $thread_attach;
                }
                DLL_THREAD_DETACH => {
                    $thread_detach;
                }
                _ => {}
            }
            TRUE
        }
    };
}

/// 関数`load`を定義するマクロ。
///
/// 引数で`bool`を返す関数名を渡してください(省略可)。
///
/// # Safety
/// このマクロで定義される関数は、指定された`HGLOBAL`ポインタを [`GlobalFree`] で解放しています。
///
/// [`GlobalFree`]: winapi::um::winbase::GlobalFree
#[macro_export]
macro_rules! define_load {
    () => {
        #[no_mangle]
        pub unsafe extern "cdecl" fn load(h: HGLOBAL, _len: c_long) -> BOOL {
            unsafe { GlobalFree(h) };

            TRUE
        }
    };

    ($load_process:ident) => {
        #[no_mangle]
        pub unsafe extern "cdecl" fn load(h: HGLOBAL, _len: c_long) -> BOOL {
            unsafe { GlobalFree(h) };

            if $load_process() {
                TRUE
            } else {
                FALSE
            }
        }
    };
}

/// 関数`request`を定義するマクロ。
///
/// 引数で、requestの内容である`&Vec<u8>`を受けとり、返答である`Vec<i8>`を返す関数名を渡してください。
///
/// # Safety
/// このマクロで定義される関数は、指定された`HGLOBAL`ポインタを [`GlobalFree`] で解放しています。
///
/// [`GlobalFree`]: winapi::um::winbase::GlobalFree
#[macro_export]
macro_rules! define_request {
    ($request_process:ident) => {
        #[no_mangle]
        pub unsafe extern "cdecl" fn request(h: HGLOBAL, len: *mut c_long) -> HGLOBAL {
            // リクエストの取得
            let s = unsafe { hglobal_to_vec_u8(h, *len) };
            unsafe { GlobalFree(h) };

            let response_bytes: Vec<i8> = $request_process(&s);

            slice_i8_to_hglobal(len, &response_bytes)
        }
    };
}

/// 関数`unload`を定義するマクロ。
///
/// 引数で`bool`を返す関数名を渡してください(省略可)。
#[macro_export]
macro_rules! define_unload {
    () => {
        #[no_mangle]
        pub extern "cdecl" fn unload() -> BOOL {
            TRUE
        }
    };

    ($unload_process:ident) => {
        #[no_mangle]
        pub extern "cdecl" fn unload() -> BOOL {
            if $unload_process() {
                TRUE
            } else {
                FALSE
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    mod slice_i8_to_hglobal {
        use super::*;

        #[test]
        fn checking_value() {
            let mut h_len_raw = 10 as c_long;
            let h_len = &mut h_len_raw as *mut c_long;
            let data = [1, 2, 3, 4];
            let result = unsafe { slice_i8_to_hglobal(h_len, &data) };

            let result_vec =
                unsafe { std::slice::from_raw_parts(result as *mut i8, *h_len as usize).to_vec() };
            unsafe { GlobalFree(result) };

            assert_eq!(result_vec, data.to_vec());
            assert_eq!(h_len_raw, 4);
        }
    }

    mod hglobal_to_vec_u8 {
        use super::*;

        #[test]
        fn checking_value() {
            let len = 4;
            let h = unsafe { GlobalAlloc(GMEM_FIXED, len) };
            let case = [1, 2, 3, 4, 0];

            let h_slice = unsafe { std::slice::from_raw_parts_mut(h as *mut u8, len) };
            h_slice.clone_from_slice(&case[..4]);

            let result = hglobal_to_vec_u8(h, len as c_long);
            unsafe { GlobalFree(h) };

            assert_eq!(result, case.to_vec());
        }
    }
}
