//! 伺かのDLL用マクロ。
//!
//! 伺かのDLLに使われる`load`と`loadu`、`request`、`unload`を定義するマクロ集です。
//!
//! Dllへのパスを返す関数 `read_dll_path_string`と、`dll_main`featureが有効のときのみ使用可能な、DLLのエントリポイントである`DllMain`を定義するマクロ`define_dll_main`も定義しています。
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
//! // v1.1.0より、DLLのパスを引数にとるようになりました。
//! fn ukagaka_load(_path: &str) -> bool {
//!     // process with dll path
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
//! // v1.1.0より、`define_dll_main`マクロを呼ばなくても、
//! // `define_load`が呼ばれていれば、DLLのパスを記録するようになりました。
//! define_load!(ukagaka_load);
//! define_request!(ukagaka_request);
//! define_unload!();
//! ```
//!
//! [`read_dll_path_string`]: crate::read_dll_path_string

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod dll_util;

pub use dll_util::read_dll_path_string;

/// 関数`DLLMain`を定義するマクロ。
///
/// 引数は順番に、`DLL_PROCESS_ATTACH`時、`DLL_PROCESS_DETACH`時、`DLL_THREAD_ATTACH`時、`DLL_THREAD_DETACH`時の処理になります。
/// それぞれ省略可で、もし、途中を飛ばしたい場合、`()`を指定してください。それでその時点での処理はなくなります。
/// 引数なしなら、何もしません。
///
/// featureの`dll_main`が有効になっていないと使用できませんが、基本的な動作には必要ありません。
#[cfg(feature = "dll_main")]
#[cfg_attr(docsrs, doc(cfg(feature = "dll_main")))]
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
            _h_module: dll_util::HINSTANCE,
            ul_reason_for_call: dll_util::DWORD,
            _l_reserved: dll_util::LPVOID,
        ) -> dll_util::BOOL {
            match ul_reason_for_call {
                dll_util::DLL_PROCESS_ATTACH => {
                    $process_attach;
                }
                dll_util::DLL_PROCESS_DETACH => {
                    $process_detach;
                }
                dll_util::DLL_THREAD_ATTACH => {
                    $thread_attach;
                }
                dll_util::DLL_THREAD_DETACH => {
                    $thread_detach;
                }
                _ => {}
            }
            dll_util::TRUE
        }
    };
}

/// 関数`load`と`loadu`を定義するマクロ。
///
/// 引数で、DLLへのパスである`&str`を受けとり、`bool`を返す関数名を渡してください。
/// 内部でDLLへのパスを記録しています。(記録したパスは[`read_dll_path_string`]で呼び出せます)
///
/// v1.1.0より、関数名は省略不可になりました。
///
/// # Safety
/// このマクロで定義される関数は、指定された`HGLOBAL`ポインタを [`global_free`] で解放しています。
///
/// [`read_dll_path_string`]: crate::read_dll_path_string
/// [`global_free`]: crate::dll_util::global_free
#[macro_export]
macro_rules! define_load {
    ($load_process:ident) => {
        #[no_mangle]
        pub unsafe extern "cdecl" fn loadu(
            h: dll_util::HGLOBAL,
            len: dll_util::c_long,
        ) -> dll_util::BOOL {
            let path_raw = dll_util::hglobal_to_vec_u8(h, len);
            unsafe { dll_util::global_free(h) };

            let path = match String::from_utf8(path_raw) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("failed to decode: {}", e);
                    return dll_util::FALSE;
                }
            };

            if let Err(e) = dll_util::register_dll_path(path.clone()) {
                eprintln!("failed to initialize dll path: {}", e);
                return dll_util::FALSE;
            }

            let result = if $load_process(&path) {
                dll_util::TRUE
            } else {
                dll_util::FALSE
            };

            if let Err(_e) = dll_util::register_loadu_result(result) {
                eprintln!("failed to record the result of loadu");
                dll_util::FALSE
            } else {
                result
            }
        }

        #[no_mangle]
        pub unsafe extern "cdecl" fn load(
            h: dll_util::HGLOBAL,
            len: dll_util::c_long,
        ) -> dll_util::BOOL {
            let path_raw = dll_util::hglobal_to_vec_u8(h, len);
            unsafe { dll_util::global_free(h) };

            // loaduの結果が記録してあったなら、それを返して処理を終了する。
            if let Some(result) = dll_util::read_loadu_result() {
                return result;
            }

            let path = match dll_util::decode_from_oem_codepage(&path_raw) {
                Ok(v) => v,
                Err(e) => return e,
            };

            if let Err(e) = dll_util::register_dll_path(path.clone()) {
                eprintln!("failed to initialize dll path: {}", e);
                return dll_util::FALSE;
            }

            if $load_process(&path) {
                dll_util::TRUE
            } else {
                dll_util::FALSE
            }
        }
    };
}

/// 関数`request`を定義するマクロ。
///
/// 引数で、requestの内容である`&Vec<u8>`を受けとり、返答である`Vec<i8>`を返す関数名を渡してください。
///
/// # Safety
/// このマクロで定義される関数は、指定された`HGLOBAL`ポインタを [`global_free`] で解放しています。
///
/// [`global_free`]: crate::dll_util::global_free
#[macro_export]
macro_rules! define_request {
    ($request_process:ident) => {
        #[no_mangle]
        pub unsafe extern "cdecl" fn request(
            h: dll_util::HGLOBAL,
            len: *mut dll_util::c_long,
        ) -> dll_util::HGLOBAL {
            // リクエストの取得
            let s = unsafe { dll_util::hglobal_to_vec_u8(h, *len) };
            unsafe { dll_util::global_free(h) };

            let response_bytes: Vec<i8> = $request_process(&s);
            dll_util::slice_i8_to_hglobal(len, &response_bytes)
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
        pub extern "cdecl" fn unload() -> dll_util::BOOL {
            dll_util::TRUE
        }
    };

    ($unload_process:ident) => {
        #[no_mangle]
        pub extern "cdecl" fn unload() -> dll_util::BOOL {
            if $unload_process() {
                dll_util::TRUE
            } else {
                dll_util::FALSE
            }
        }
    };
}
