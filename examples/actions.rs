#![allow(unused_imports)]
use notify_rust::{Hint, Notification};
#[cfg(windows)]
use notify_rust::Action;

#[cfg(windows)]
fn main() {
    Notification::new()
        .summary("click me")
        .action("default", "default")    // IDENTIFIER, LABEL
        .action("clicked_a", "button a") // IDENTIFIER, LABEL
        .action("clicked_b", "button b") // IDENTIFIER, LABEL
        .show()
        .unwrap()
        .wait_for_action(|action|
            match action {
                Action::Activate(val) => match &*val {
                    "default"  => println!("default"),
                    "clicked_a"  => println!("clicked a"),
                    "clicked_b"  => println!("clicked b"),
                    _ => ()
                }
                Action::Close => println!("the notification was closed"),
                Action::Fail => println!("notification failure"),
                _ => ()
            }
        );
}

#[cfg(unix)]
fn main() {
    #[cfg(all(unix, not(target_os = "macos")))]
    Notification::new()
        .summary("click me")
        .action("default", "default")    // IDENTIFIER, LABEL
        .action("clicked_a", "button a") // IDENTIFIER, LABEL
        .action("clicked_b", "button b") // IDENTIFIER, LABEL
        .hint(Hint::Resident(true))
        .show()
        .unwrap()
        .wait_for_action(|action|
            match action {
                "default"  => println!("default"),
                "clicked_a"  => println!("clicked a"),
                "clicked_b"  => println!("clicked b"),
                // FIXME: here "__closed" is a hardcoded keyword, it will be deprecated!!
                "__closed" => println!("the notification was closed"),
                _ => ()
            }
        );

    #[cfg(target_os = "macos")]
    Notification::new()
        .summary("PLATFORM ERROR")
        .subtitle("unsupported functionality")
        .body("cannot wait for closing on macOS.")
        .show()
        .unwrap();
}
