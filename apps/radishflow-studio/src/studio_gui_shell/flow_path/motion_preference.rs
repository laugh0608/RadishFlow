#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub(super) enum MotionPreference {
    Allowed,
    Reduced,
    Unknown,
}

#[cfg(not(target_os = "macos"))]
pub(super) fn system_motion_preference() -> MotionPreference {
    MotionPreference::Unknown
}

#[cfg(target_os = "macos")]
pub(super) fn system_motion_preference() -> MotionPreference {
    use std::ffi::{c_char, c_int, c_void};

    type Object = *mut c_void;
    type Selector = *mut c_void;

    #[cfg(target_arch = "aarch64")]
    type ObjcBool = bool;
    #[cfg(not(target_arch = "aarch64"))]
    type ObjcBool = i8;

    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {}

    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Object;
        fn sel_registerName(name: *const c_char) -> Selector;
        fn objc_msgSend();
    }

    unsafe extern "C" {
        fn pthread_main_np() -> c_int;
    }

    // NSWorkspace and its accessibility options are AppKit reads on the GUI thread.
    // The two function signatures match the SDK's class method and BOOL getter.
    unsafe {
        if pthread_main_np() == 0 {
            return MotionPreference::Unknown;
        }
        let class = objc_getClass(c"NSWorkspace".as_ptr());
        if class.is_null() {
            return MotionPreference::Unknown;
        }
        let shared_workspace = std::mem::transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(Object, Selector) -> Object,
        >(objc_msgSend);
        let workspace = shared_workspace(class, sel_registerName(c"sharedWorkspace".as_ptr()));
        if workspace.is_null() {
            return MotionPreference::Unknown;
        }
        let option = std::mem::transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(Object, Selector) -> ObjcBool,
        >(objc_msgSend);
        if option(
            workspace,
            sel_registerName(c"accessibilityDisplayShouldReduceMotion".as_ptr()),
        ) as u8
            != 0
        {
            MotionPreference::Reduced
        } else {
            MotionPreference::Allowed
        }
    }
}
