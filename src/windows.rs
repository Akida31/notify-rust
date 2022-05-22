use winrt_notification::{Toast, Action as WinAction, Header, ToastWithHandlers};

pub use crate::{
    error::*,
    notification::Notification,
    timeout::Timeout,
};


use std::{path::Path, str::FromStr, sync::mpsc};

#[derive(Debug)]
#[non_exhaustive]
/// Action created by interaction with the notification
pub enum Action {
    /// Occurs when user activates a toast notification through a click or touch.
    Activate(String),
    /// Occurs when a toast notification leaves the screen, either by expiring or being explicitly dismissed by the user.
    Close,
    /// Occurs when an error is caused when Windows attempts to raise a toast notification.
    Fail,
}

/// A handle to a shown notification.
#[derive(Debug)]
pub struct NotificationHandle {
    action_receiver: mpsc::Receiver<Action>,
}

impl NotificationHandle {
    /// Waits for the user to act on a notification and then calls
    /// `invocation_closure` with the corresponding [`Action`].
    pub fn wait_for_action<F>(self, invocation_closure: F)
        where F: FnOnce(Action), {
            invocation_closure(self.action_receiver.recv().unwrap())
        }
}

pub(crate) fn show_notification(notification: &Notification) -> Result<NotificationHandle> {
    let sound = match &notification.sound_name {
        Some(chosen_sound_name) => winrt_notification::Sound::from_str(chosen_sound_name).ok(),
        None => None,
    };

    let duration = match notification.timeout {
        Timeout::Default => winrt_notification::Duration::Short,
        Timeout::Never => winrt_notification::Duration::Long,
        Timeout::Milliseconds(t) => {
            if t >= 25000 {
                winrt_notification::Duration::Long
            } else {
                winrt_notification::Duration::Short
            }
        }
    };

    let powershell_app_id = &Toast::POWERSHELL_APP_ID.to_string();
    let app_id = &notification.app_id.as_ref().unwrap_or(powershell_app_id);
    let mut toast = Toast::new(app_id)
            .title(&notification.summary)
            .text1(notification.subtitle.as_ref().map(AsRef::as_ref).unwrap_or("")) // subtitle
            .text2(&notification.body)
            .sound(sound)
            .duration(duration);
    for action in notification.actions.chunks(2) {
        let identifier = action[0].clone();
        let label = action[1].clone();
        toast = toast.action(WinAction {
            arguments: identifier,
            content: label,
            ..Default::default()
        });
    }
    if let Some(image_path) = &notification.path_to_image {
        toast = toast.image(Path::new(&image_path), "");
    }

    if let Some(header) = &notification.header {
        toast = toast.header(Header::from_title(header));
    }

    let mut toast = ToastWithHandlers::new(toast);
    let (action_sender, action_receiver) = mpsc::channel();
    if !notification.actions.is_empty() {
        let sender = action_sender.clone();
        toast = toast.on_activate(move |event| {
            let arg = event.get_arguments().unwrap()?;
            sender.send(Action::Activate(arg)).unwrap();
            Ok(())
        });
        let sender = action_sender.clone();
        toast = toast.on_dismiss(move |_, _| {
            sender.send(Action::Close).unwrap();
            Ok(())
        });
        toast = toast.on_fail(move |_, _| {
            action_sender.send(Action::Fail).unwrap();
            Ok(())
        });
    }

    toast
        .show()
        .map_err(|e| Error::from(ErrorKind::Msg(format!("{:?}", e))))?;
    Ok(NotificationHandle {action_receiver})
}
