// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Memory allocation APIs

#![stable(feature = "alloc_module", since = "1.28.0")]

use core::sync::atomic::{AtomicPtr, Ordering};
use core::{mem, ptr};

#[stable(feature = "alloc_module", since = "1.28.0")]
#[doc(inline)]
pub use alloc_crate::alloc::*;

#[stable(feature = "alloc_system_type", since = "1.28.0")]
#[doc(inline)]
pub use alloc_system::System;

static HOOK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

/// Registers a custom OOM hook, replacing any that was previously registered.
///
/// The OOM hook is invoked when an infallible memory allocation fails.
/// The default hook prints a message to standard error and aborts the
/// execution, but this behavior can be customized with the [`set_oom_hook`]
/// and [`take_oom_hook`] functions.
///
/// The hook is provided with a `Layout` struct which contains information
/// about the allocation that failed.
///
/// The OOM hook is a global resource.
#[unstable(feature = "oom_hook", issue = "51245")]
pub fn set_oom_hook(hook: fn(Layout) -> !) {
    HOOK.store(hook as *mut (), Ordering::SeqCst);
}

/// Unregisters the current OOM hook, returning it.
///
/// *See also the function [`set_oom_hook`].*
///
/// If no custom hook is registered, the default hook will be returned.
#[unstable(feature = "oom_hook", issue = "51245")]
pub fn take_oom_hook() -> fn(Layout) -> ! {
    let hook = HOOK.swap(ptr::null_mut(), Ordering::SeqCst);
    if hook.is_null() {
        default_oom_hook
    } else {
        unsafe { mem::transmute(hook) }
    }
}

fn default_oom_hook(layout: Layout) -> ! {
    rtabort!("memory allocation of {} bytes failed", layout.size())
}

#[cfg(not(test))]
#[doc(hidden)]
#[lang = "oom"]
#[unstable(feature = "alloc_internals", issue = "0")]
pub extern fn rust_oom(layout: Layout) -> ! {
    let hook = HOOK.load(Ordering::SeqCst);
    let hook: fn(Layout) -> ! = if hook.is_null() {
        default_oom_hook
    } else {
        unsafe { mem::transmute(hook) }
    };
    hook(layout)
}

#[cfg(not(test))]
#[doc(hidden)]
#[allow(unused_attributes)]
#[unstable(feature = "alloc_internals", issue = "0")]
pub mod __default_lib_allocator {
    use super::{System, Layout, GlobalAlloc};
    // for symbol names src/librustc/middle/allocator.rs
    // for signatures src/librustc_allocator/lib.rs

    // linkage directives are provided as part of the current compiler allocator
    // ABI

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_alloc(size: usize, align: usize) -> *mut u8 {
        let layout = Layout::from_size_align_unchecked(size, align);
        System.alloc(layout)
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_dealloc(ptr: *mut u8,
                                       size: usize,
                                       align: usize) {
        System.dealloc(ptr, Layout::from_size_align_unchecked(size, align))
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_realloc(ptr: *mut u8,
                                       old_size: usize,
                                       align: usize,
                                       new_size: usize) -> *mut u8 {
        let old_layout = Layout::from_size_align_unchecked(old_size, align);
        System.realloc(ptr, old_layout, new_size)
    }

    #[no_mangle]
    #[rustc_std_internal_symbol]
    pub unsafe extern fn __rdl_alloc_zeroed(size: usize, align: usize) -> *mut u8 {
        let layout = Layout::from_size_align_unchecked(size, align);
        System.alloc_zeroed(layout)
    }
}
