mod direction;
mod keyboard;
mod mode;
mod terminal;

use keyboard::handle_event;
use terminal::Terminal;

type TerminalResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> TerminalResult<()> {
	let mut terminal = Terminal::new()?;

	loop {
		terminal.print_buffer()?;

		if handle_event(&mut terminal)? {
			break;
		}
	}

	Ok(())
}
