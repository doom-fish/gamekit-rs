use crate::{private, GameKitError};

/// Convenience wrapper around `GKNotificationBanner`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotificationBanner;

impl NotificationBanner {
    /// Shows a notification banner using the default duration.
    pub fn show(title: Option<&str>, message: Option<&str>) -> Result<(), GameKitError> {
        let title = title.map(|value| private::cstring_from_str(value, "notification title"));
        let message = message.map(|value| private::cstring_from_str(value, "notification message"));
        let title = title.transpose()?;
        let message = message.transpose()?;

        unsafe {
            crate::ffi::gk_notification_show(
                title.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),
                message.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),
            );
        }
        Ok(())
    }

    /// Shows a notification banner for the supplied duration.
    pub fn show_for_duration(
        title: Option<&str>,
        message: Option<&str>,
        duration_seconds: f64,
    ) -> Result<(), GameKitError> {
        let title = title.map(|value| private::cstring_from_str(value, "notification title"));
        let message = message.map(|value| private::cstring_from_str(value, "notification message"));
        let title = title.transpose()?;
        let message = message.transpose()?;

        unsafe {
            crate::ffi::gk_notification_show_with_duration(
                title.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),
                message.as_ref().map_or(std::ptr::null(), |value| value.as_ptr()),
                duration_seconds,
            );
        }
        Ok(())
    }
}
