//! Small wrapper over `arboard` for clipboard funtionality.

use arboard::Clipboard;

use crate::Result;

/// Get text from the system clipboard.
///
/// Return an error if it fails to get the systems clipboard.
pub fn get_text() -> Result<Option<String>> {
	let mut clipboard = Clipboard::new()?;

	Ok(clipboard.get_text().ok())
}

#[allow(dead_code)]
// Will remain dead code until selection is implemented
pub fn set_text(text: &str) -> Result<()> {
	let mut clipboard = Clipboard::new()?;
	clipboard.set_text(text)?;

	Ok(())
}
