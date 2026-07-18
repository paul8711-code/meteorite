use native_dialog::{DialogBuilder, MessageLevel};

// helper for displaying a dialog message (usually indicating that something went wrong)
pub fn show_dialog_window(title: impl ToString, text: impl ToString, level: MessageLevel) {
    DialogBuilder::message()
        .set_title(title)
        .set_text(text)
        .set_level(level)
        .alert()
        .show()
        .unwrap();
}
