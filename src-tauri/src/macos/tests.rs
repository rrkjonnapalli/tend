use super::*;
use objc2::runtime::AnyClass;

#[test]
fn delegate_answers_should_present() {
    show_notifications_in_foreground();
    let class = AnyClass::get(DELEGATE).expect("mac-notification-sys delegate class is linked");
    assert!(class
        .instance_method(sel!(userNotificationCenter:shouldPresentNotification:))
        .is_some());
}
