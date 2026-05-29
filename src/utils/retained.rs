//! Declarative macro for retain/release wrapper boilerplate.
//!
//! Many wrapper types hold a single `*mut c_void` pointer to a retained
//! Objective-C / Swift object and hand-roll identical `Clone` (retain) and
//! `Drop` (release) implementations. `gk_retained!` consolidates that
//! boilerplate into a single audited place.
//!
//! The generated impls preserve the exact behavior of the previous
//! hand-written versions:
//! - `Clone` bumps the retain count by calling the supplied `retain` FFI fn.
//! - `Drop` null-checks the pointer before calling the supplied `release`
//!   FFI fn.
//!
//! Types whose `Clone`/`Drop` carry extra logic beyond retain/release +
//! null-check (e.g. delegate guards that also clear callbacks and free a
//! boxed handler) are intentionally left hand-written.

/// Generate `Clone` and/or `Drop` impls for a retain/release pointer wrapper.
///
/// Variants:
/// - Named-field struct (`{ ptr }`), full `Clone` + `Drop`:
///   `gk_retained!(Ty, field = ptr, retain = path::retain, release = path::release);`
/// - Named-field struct, drop-only:
///   `gk_retained!(Ty, field = ptr, release = path::release);`
macro_rules! gk_retained {
    // Named-field struct: Clone + Drop
    ($ty:ty, field = $field:ident, retain = $retain:path, release = $release:path $(,)?) => {
        impl Clone for $ty {
            fn clone(&self) -> Self {
                Self {
                    $field: unsafe { $retain(self.$field) },
                }
            }
        }

        impl Drop for $ty {
            fn drop(&mut self) {
                if !self.$field.is_null() {
                    unsafe {
                        $release(self.$field);
                    }
                }
            }
        }
    };

    // Named-field struct: Drop only
    ($ty:ty, field = $field:ident, release = $release:path $(,)?) => {
        impl Drop for $ty {
            fn drop(&mut self) {
                if !self.$field.is_null() {
                    unsafe {
                        $release(self.$field);
                    }
                }
            }
        }
    };
}

pub(crate) use gk_retained;
