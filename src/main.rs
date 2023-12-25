mod direction;
mod keyboard;
mod mode;
mod terminal;

use keyboard::handle_input;
use terminal::Terminal;

type TerminalResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> TerminalResult<()> {
	let mut terminal = Terminal::new()?;

	loop {
		terminal.print_buffer().expect("Failed to print buffer");

		if handle_input(&mut terminal)? {
			break;
		}
	}

	Ok(())
}
