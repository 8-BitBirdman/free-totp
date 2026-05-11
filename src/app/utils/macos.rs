#[cfg(target_os = "macos")]
pub fn set_activation_policy_accessory() {
    use objc::{msg_send, sel, sel_impl};
    
    // Safety: Talking to NSApplication is standard for macOS GUI apps to control Dock behavior.
    unsafe {
        let cls = objc::runtime::Class::get("NSApplication").unwrap();
        let app: *mut objc::runtime::Object = msg_send![cls, sharedApplication];
        if !app.is_null() {
            // NSApplicationActivationPolicyAccessory = 1
            // This hides the app from the Dock but keeps the window capabilities.
            let _: () = msg_send![app, setActivationPolicy: 1isize];
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_activation_policy_accessory() {}
