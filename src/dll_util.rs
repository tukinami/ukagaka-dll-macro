//! マクロ以外の関数や型など。
use std::sync::OnceLock;

static DLL_PATH: OnceLock<String> = OnceLock::new();
static LOADU_RESULT: OnceLock<BOOL> = OnceLock::new();

use encoding::{label::encoding_from_windows_code_page, DecoderTrap};
use winapi::um::{
    winbase::{GlobalAlloc, GlobalFree, GMEM_FIXED},
    winnls::GetOEMCP,
};

pub use std::ffi::c_long;
pub use winapi::shared::minwindef::{BOOL, FALSE, HGLOBAL, TRUE};

#[cfg(feature = "dll_main")]
pub use winapi::{
    shared::minwindef::{DWORD, HINSTANCE, LPVOID},
    um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH},
};

/// DLLへのパスを返す関数。
///
/// [`define_load`]で定義される`load`か`loadu`時にDLLへのパスが記録されます。
///
/// [`define_load`]: crate::define_load
pub fn read_dll_path_string() -> Option<String> {
    DLL_PATH.get().cloned()
}

/// DLLへのパスを記録する関数。
///
/// [`define_load`]で定義される`load`か`loadu`時に、この関数が呼ばれます。
///
/// [`define_load`]: crate::define_load
pub fn register_dll_path(path: String) -> Result<(), String> {
    DLL_PATH.set(path)
}

/// `loadu`の結果を返す関数。
///
/// [`define_load`]で定義されるloadu`時に結果が記録されます。
///
/// [`define_load`]: crate::define_load
pub fn read_loadu_result() -> Option<BOOL> {
    LOADU_RESULT.get().cloned()
}

/// `loadu`の結果を記録する関数。
///
/// [`define_load`]で定義される`loadu`時に、この関数が呼ばれます。
///
/// [`define_load`]: crate::define_load
pub fn register_loadu_result(result: BOOL) -> Result<(), BOOL> {
    LOADU_RESULT.set(result)
}

/// `u8`のスライスを、OEM codepageで`String`にデコードする関数。
pub fn decode_from_oem_codepage(bytes: &[u8]) -> Result<String, BOOL> {
    let oem_codepage = unsafe { GetOEMCP() };
    let encoding = match encoding_from_windows_code_page(oem_codepage as usize) {
        Some(v) => v,
        None => {
            eprintln!("unsupport OEM codepage: {}", oem_codepage);
            return Err(FALSE);
        }
    };
    match encoding.decode(bytes, DecoderTrap::Strict) {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!("failed to decode: {}", e);
            Err(FALSE)
        }
    }
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

/// `GlobalFree`を呼ぶ関数。
///
/// # Safety
/// この関数は、指定された`HGLOBAL`ポインタを [`GlobalFree`] で解放しています。
///
/// [`GlobalFree`]: winapi::um::winbase::GlobalFree
pub unsafe fn global_free(h: HGLOBAL) {
    unsafe { GlobalFree(h) };
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

        use winapi::um::winbase::{GlobalAlloc, GMEM_FIXED};

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
