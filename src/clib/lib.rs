// SPDX-License-Identifier: Apache-2.0

mod logger;

use std::ffi::CString;
use std::os::raw::c_char;
use std::time::SystemTime;

use once_cell::sync::OnceCell;
use rabc::{ErrorKind, RabcClient, RabcError, RabcEvent};

use crate::logger::MemoryLogger;

const RABC_PASS: u32 = 0;
const RABC_FAIL: u32 = 1;
const RABC_FAIL_NULL_POINTER: u32 = 2;

static INSTANCE: OnceCell<MemoryLogger> = OnceCell::new();

#[no_mangle]
pub extern "C" fn rabc_client_new(
    client: *mut *mut RabcClient,
    log: *mut *mut c_char,
    err_kind: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> u32 {
    if client.is_null()
        || log.is_null()
        || err_kind.is_null()
        || err_msg.is_null()
    {
        return RABC_FAIL_NULL_POINTER;
    }

    unsafe {
        *client = std::ptr::null_mut();
        *log = std::ptr::null_mut();
        *err_kind = std::ptr::null_mut();
        *err_msg = std::ptr::null_mut();
    }

    let logger = match init_logger() {
        Ok(l) => l,
        Err(e) => {
            unsafe {
                *err_msg =
                    CString::new(format!("Failed to setup logger: {}", e))
                        .unwrap()
                        .into_raw();
            }
            return RABC_FAIL;
        }
    };
    let now = SystemTime::now();

    let result = RabcClient::new();

    unsafe {
        *log = CString::new(logger.drain(now)).unwrap().into_raw();
    }

    match result {
        Ok(c) => unsafe {
            *client = Box::into_raw(Box::new(c));
            RABC_PASS
        },
        Err(e) => unsafe {
            *err_msg = CString::new(e.msg()).unwrap().into_raw();
            *err_kind =
                CString::new(format!("{}", &e.kind())).unwrap().into_raw();
            RABC_FAIL
        },
    }
}

#[no_mangle]
pub extern "C" fn rabc_client_poll(
    client: *mut RabcClient,
    wait_time: u32,
    events: *mut *mut u64,
    event_count: *mut u64,
    log: *mut *mut c_char,
    err_kind: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> u32 {
    if client.is_null()
        || events.is_null()
        || event_count.is_null()
        || log.is_null()
        || err_kind.is_null()
        || err_msg.is_null()
    {
        return RABC_FAIL_NULL_POINTER;
    }

    unsafe {
        *event_count = 0;
        *events = std::ptr::null_mut();
        *log = std::ptr::null_mut();
        *err_kind = std::ptr::null_mut();
        *err_msg = std::ptr::null_mut();
    }

    let client: &mut RabcClient = unsafe { &mut *client };

    let logger = match init_logger() {
        Ok(l) => l,
        Err(e) => {
            unsafe {
                *err_msg =
                    CString::new(format!("Failed to setup logger: {}", e))
                        .unwrap()
                        .into_raw();
            }
            return RABC_FAIL;
        }
    };
    let now = SystemTime::now();

    let result = client.poll(wait_time);

    unsafe {
        *log = CString::new(logger.drain(now)).unwrap().into_raw();
    }

    match result {
        Ok(result_events) => {
            if !result_events.is_empty() {
                let result_events: Vec<u64> = result_events
                    .as_slice()
                    .iter()
                    .map(|e| *e as u64)
                    .collect();
                let event_ids_len = result_events.len() as u64;
                // We trust C library user to use `rabc_events_free()`
                let mut event_ids_box = result_events.into_boxed_slice();
                unsafe {
                    *event_count = event_ids_len;
                    *events = event_ids_box.as_mut_ptr();
                }
                std::mem::forget(event_ids_box);
            }
            RABC_PASS
        }
        Err(e) => unsafe {
            *err_msg = CString::new(e.msg()).unwrap().into_raw();
            *err_kind =
                CString::new(format!("{}", &e.kind())).unwrap().into_raw();
            RABC_FAIL
        },
    }
}

#[no_mangle]
pub extern "C" fn rabc_client_process(
    client: *mut RabcClient,
    event: u64,
    reply: *mut *mut c_char,
    log: *mut *mut c_char,
    err_kind: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> u32 {
    if client.is_null()
        || reply.is_null()
        || log.is_null()
        || err_kind.is_null()
        || err_msg.is_null()
    {
        return RABC_FAIL_NULL_POINTER;
    }

    unsafe {
        *reply = std::ptr::null_mut();
        *log = std::ptr::null_mut();
        *err_kind = std::ptr::null_mut();
        *err_msg = std::ptr::null_mut();
    }

    let client: &mut RabcClient = unsafe { &mut *client };

    let logger = match init_logger() {
        Ok(l) => l,
        Err(e) => {
            unsafe {
                *err_msg =
                    CString::new(format!("Failed to setup logger: {}", e))
                        .unwrap()
                        .into_raw();
            }
            return RABC_FAIL;
        }
    };
    let now = SystemTime::now();

    let event = match RabcEvent::try_from(event) {
        Ok(e) => e,
        Err(e) => {
            unsafe {
                *err_msg = CString::new(e.msg()).unwrap().into_raw();
                *err_kind =
                    CString::new(format!("{}", &e.kind())).unwrap().into_raw();
            }
            return RABC_FAIL;
        }
    };

    let result = client.process(&event);
    unsafe {
        *log = CString::new(logger.drain(now)).unwrap().into_raw();
    }

    match result {
        Ok(Some(r)) => {
            if !r.is_empty() {
                unsafe {
                    *reply = CString::new(r).unwrap().into_raw();
                }
            }
            RABC_PASS
        }
        Ok(None) => RABC_PASS,
        Err(e) => unsafe {
            *err_msg = CString::new(e.msg()).unwrap().into_raw();
            *err_kind =
                CString::new(format!("{}", &e.kind())).unwrap().into_raw();
            RABC_FAIL
        },
    }
}

#[no_mangle]
pub extern "C" fn rabc_client_free(client: *mut RabcClient) {
    if !client.is_null() {
        unsafe {
            drop(Box::from_raw(client));
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn rabc_cstring_free(cstring: *mut c_char) {
    unsafe {
        if !cstring.is_null() {
            drop(CString::from_raw(cstring));
        }
    }
}

#[no_mangle]
pub extern "C" fn rabc_events_free(events: *mut u64, event_count: u64) {
    unsafe {
        if !events.is_null() {
            let events_slice =
                std::slice::from_raw_parts_mut(events, event_count as usize);
            drop(Box::from_raw(events_slice));
        }
    }
}

fn init_logger() -> Result<&'static MemoryLogger, RabcError> {
    match INSTANCE.get() {
        Some(l) => {
            l.add_consumer();
            Ok(l)
        }
        None => {
            if INSTANCE.set(MemoryLogger::new()).is_err() {
                return Err(RabcError::new(
                    ErrorKind::Bug,
                    "Failed to set once_sync for logger".to_string(),
                ));
            }
            if let Some(l) = INSTANCE.get() {
                if let Err(e) = log::set_logger(l) {
                    Err(RabcError::new(
                        ErrorKind::Bug,
                        format!("Failed to log::set_logger: {}", e),
                    ))
                } else {
                    l.add_consumer();
                    log::set_max_level(log::LevelFilter::Debug);
                    Ok(l)
                }
            } else {
                Err(RabcError::new(
                    ErrorKind::Bug,
                    "Failed to get logger from once_sync".to_string(),
                ))
            }
        }
    }
}
