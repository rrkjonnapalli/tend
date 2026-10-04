//! macOS only. Notifications go through `mac-notification-sys`, whose delegate doesn't answer
//! `userNotificationCenter:shouldPresentNotification:`. macOS then hides the banner whenever
//! Tend is the frontmost app. This adds that method (always YES), so a timer that ends while
//! you're looking at Tend still shows its banner.

#[cfg(test)]
mod tests;

use std::ffi::{c_char, CString};

use objc2::encode::Encode;
use objc2::ffi::class_addMethod;
use objc2::runtime::{AnyClass, AnyObject, Bool, Imp, Sel};
use objc2::sel;

/// The delegate class compiled into the binary by mac-notification-sys.
const DELEGATE: &std::ffi::CStr = c"NotificationCenterDelegate";

unsafe extern "C-unwind" fn should_present(
    _this: *mut AnyObject,
    _cmd: Sel,
    _center: *mut AnyObject,
    _note: *mut AnyObject,
) -> Bool {
    Bool::YES
}

/// Call once at startup, before the first notification.
pub fn show_notifications_in_foreground() {
    let Some(class) = AnyClass::get(DELEGATE) else {
        return;
    };
    // Method signature: BOOL (id self, SEL _cmd, id center, id notification).
    let Ok(types) = CString::new(format!("{}@:@@", Bool::ENCODING)) else {
        return;
    };
    let imp: unsafe extern "C-unwind" fn(
        *mut AnyObject,
        Sel,
        *mut AnyObject,
        *mut AnyObject,
    ) -> Bool = should_present;
    // SAFETY: `imp` matches the encoded signature, and the class lives for the whole process.
    // class_addMethod does nothing if the class already has the method.
    unsafe {
        let imp: Imp = std::mem::transmute(imp);
        class_addMethod(
            class as *const AnyClass as *mut AnyClass,
            sel!(userNotificationCenter:shouldPresentNotification:),
            imp,
            types.as_ptr() as *const c_char,
        );
    }
}
