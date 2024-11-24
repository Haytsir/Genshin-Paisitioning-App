pub fn confirm_dialog(title:&str, desc: &str, is_error: bool) -> bool {
    #[cfg(not(feature = "gtk3"))]
    let res = "";
    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        all(
            any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "netbsd",
                target_os = "openbsd"
            ),
            feature = "gtk3"
        )
    ))]
    let res = rfd::MessageDialog::new()
        .set_title(title)
        .set_description(desc)
        .set_buttons(rfd::MessageButtons::Ok)
        .set_level(if is_error{rfd::MessageLevel::Error}else{rfd::MessageLevel::Info})
        .show();
}