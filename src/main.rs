mod direction;
mod mode;
mod terminal;

use terminal::Terminal;

/// Result of an operation interfacing with the terminal.
///
/// Errors are generally caused by an issue within crossterm or the terminal.
type TerminalResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> TerminalResult<()> {
	let mut terminal = Terminal::new()?;

	while terminal.is_running() {
		terminal.print_buffer()?;

		terminal.handle_event()?;
	}

	Ok(())
}
