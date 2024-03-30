/// Messages sent from the user to the application
#[derive(Clone, Debug)]
pub enum Message {
    /// Quit the application
    Quit,
    /// Switch to select screen
    SwitchSelect,
    /// Switches to displaying the (markdown) file at the given path
    SwitchDisplay(crate::data::Note),
}
