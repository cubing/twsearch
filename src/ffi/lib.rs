mod events;

use std::{
    ffi::{c_char, CStr, CString},
    ptr::null_mut,
    str::FromStr,
};

use twips::scramble::{
    derive_scramble_for_event_seeded, random_scramble_for_event,
    scramble_finder::free_memory_for_all_scramble_finders, DerivationSalt, DerivationSeed, Event,
};

fn unwrap_cstr_result_or_null_ptr(result: Result<*const c_char, ()>) -> *const c_char {
    result.unwrap_or(null_mut())
}

fn war_cstr_to_rust_str_ref<'a>(cstr: *const c_char) -> Result<&'a str, ()> {
    let cstr = unsafe { CStr::from_ptr(cstr) };
    cstr.to_str().map_err(|_| ())
}

// TODO: we can't avoid leaking the return value, but we could give a function to free all past returned values.
fn rust_str_to_raw_cstr(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

/// # Safety
///
/// This function can panic. If you are working in pure Rust, use [`twips::scramble::random_scramble_for_event`] instead.
///
/// Returns:
/// - A null pointer for *any* error.
/// - A valid scramble (in the form of a C string) otherwise.
#[no_mangle]
pub unsafe extern "C" fn ffi_random_scramble_for_event(
    event_raw_cstr: *const c_char,
) -> *const c_char {
    unwrap_cstr_result_or_null_ptr(ffi_random_scramble_for_event_internal(event_raw_cstr))
}

fn ffi_random_scramble_for_event_internal(
    event_raw_cstr: *const c_char,
) -> Result<*const c_char, ()> {
    let event_str = war_cstr_to_rust_str_ref(event_raw_cstr)?;
    let event = Event::try_from(event_str).map_err(|_| ())?;
    let result_str = random_scramble_for_event(event)
        .map_err(|_| ())?
        .to_string();
    Ok(rust_str_to_raw_cstr(&result_str))
}

/// # Safety
///
/// This function can panic. If you are working in pure Rust, use [`twips::scramble::derive_scramble_for_event`] instead.
///
/// Returns:
/// - A null pointer for *any* error.
/// - A valid derived scramble (in the form of a C string) otherwise.
#[no_mangle]
pub extern "C" fn ffi_derive_scramble_for_event(
    hex_derivation_seed_cstr: *const c_char,
    // Blank string or a slash-separated hierarchy
    derivation_salt_hierarchy_str: *const c_char,
    subevent_str: *const c_char,
) -> *const c_char {
    unwrap_cstr_result_or_null_ptr(ffi_derive_scramble_for_event_internal(
        hex_derivation_seed_cstr,
        derivation_salt_hierarchy_str,
        subevent_str,
    ))
}

fn ffi_derive_scramble_for_event_internal(
    hex_derivation_seed_raw_cstr: *const c_char,
    // Blank string or a slash-separated hierarchy
    derivation_salt_hierarchy_raw_cstr: *const c_char,
    subevent_raw_cstr: *const c_char,
) -> Result<*const c_char, ()> {
    let hex_derivation_seed_str = war_cstr_to_rust_str_ref(hex_derivation_seed_raw_cstr)?;
    let derivation_salt_hierarchy_str =
        war_cstr_to_rust_str_ref(derivation_salt_hierarchy_raw_cstr)?;
    let subevent_str = war_cstr_to_rust_str_ref(subevent_raw_cstr)?;

    let derivation_seed = DerivationSeed::from_str(hex_derivation_seed_str).map_err(|_| ())?;
    let hierarchy = if derivation_salt_hierarchy_str.is_empty() {
        vec![]
    } else {
        derivation_salt_hierarchy_str
            .split("/")
            .map(DerivationSalt::from_str)
            .collect::<Result<Vec<DerivationSalt>, String>>()
            .map_err(|_| ())?
    };
    let subevent = Event::try_from(subevent_str)
        .map_err(|e| e.description)
        .map_err(|_| ())?;
    match derive_scramble_for_event_seeded(&derivation_seed, &hierarchy, subevent) {
        Ok(scramble) => Ok(rust_str_to_raw_cstr(&scramble.to_string())),
        Err(_) => Err(()),
    }
}

#[no_mangle]
pub extern "C" fn ffi_free_memory_for_all_scramble_finders() -> u32 {
    // We cast to `u32` for the public API so that it's more stable across environments (including WASM).
    // If we've allocated more than `u32::MAX` scramble finders, I'd be *very* impressed.
    free_memory_for_all_scramble_finders() as u32
}

#[test]
fn ffi_test() {
    // event ID, min num moves (inclusive), max num moves (inclusive)
    let test_data = [
        ("222", 11, 13), // TODO: are there any states that can't be reached in exactly 11 moves for our scramble generators?
        ("pyram", 11, 15), // TODO: are there any states that can't be reached in exactly 11 moves for our scramble generators (ignoring tips)?
        ("333", 15, 30),
        ("555", 60, 60),
        ("666", 80, 80),
        ("777", 100, 100),
        ("minx", 83, 83),
    ];

    let dylib_path = test_cdylib::build_current_project();
    let lib = unsafe { libloading::Library::new(dylib_path).unwrap() };
    let func: libloading::Symbol<unsafe extern "C" fn(event_raw_cstr: *mut c_char) -> *mut c_char> =
        unsafe { lib.get(b"ffi_random_scramble_for_event").unwrap() };
    for (event_id, min_num_moves, max_num_moves) in test_data {
        let event_raw_cstr = CString::new((event_id).to_owned()).unwrap().into_raw();
        let scramble_raw_cstr = unsafe { func(event_raw_cstr) };
        let scramble_cstr = unsafe { CStr::from_ptr(scramble_raw_cstr) };
        let scramble_str = scramble_cstr.to_str().map_err(|_| ()).unwrap();
        let alg = scramble_str.parse::<cubing::alg::Alg>().unwrap();
        assert!(alg.nodes.len() >= min_num_moves);
        assert!(alg.nodes.len() <= max_num_moves);
    }
}
