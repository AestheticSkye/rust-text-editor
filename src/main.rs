mod direction;
mod mode;
mod terminal;

use terminal::Terminal;

type TerminalResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> TerminalResult<()> {
	let mut terminal = Terminal::new()?;

	loop {
		terminal.print_buffer()?;

		if terminal.handle_event()? {
			break;
		}
	}

	Ok(())
}
