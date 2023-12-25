use std::io::{stdout, Write};

use crossterm::cursor::{
	MoveDown, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveToNextLine, MoveToRow, MoveUp,
	RestorePosition, SavePosition, Show,
};
use crossterm::style::{Print, ResetColor, SetBackgroundColor};
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
	LeaveAlternateScreen,
};
use crossterm::{execute, queue};

use crate::direction::Direction;
use crate::mode::Mode;
use crate::TerminalResult;

pub struct Terminal {
	columns: u16,
	rows: u16,
	current_column: usize,
	current_row: usize,
	buffer: Vec<Vec<char>>,
	output: Box<dyn std::io::Write>,
	pub mode: Mode,
}

impl Drop for Terminal {
	fn drop(&mut self) {
		disable_raw_mode().expect("Failed to disable raw mode");
		execute!(self.output, ResetColor, Show, LeaveAlternateScreen)
			.expect("Failed to clean up screen");
	}
}

#[allow(clippy::cast_possible_truncation)]
impl Terminal {
	pub fn new() -> TerminalResult<Self> {
		execute!(stdout(), EnterAlternateScreen)?;
		enable_raw_mode()?;

		// Register a panic hook to run the drop behavior if the program panics.
		// Required as panic errors will not get printed otherwise.
		std::panic::set_hook(Box::new(|panic_info| {
			disable_raw_mode().expect("Failed to disable raw mode");
			execute!(stdout(), ResetColor, Show, LeaveAlternateScreen)
				.expect("Failed to clean up screen");
			println!("{panic_info}");
		}));

		let (columns, rows) = size()?;

		Ok(Self {
			buffer: vec![vec![]],
			columns,
			rows,
			mode: Mode::Normal,
			current_column: 0,
			current_row: 0,
			output: Box::new(stdout()),
		})
	}

	pub fn insert_char(&mut self, char: char) -> TerminalResult<()> {
		self.buffer[self.current_row].insert(self.current_column, char);

		self.current_column += 1;

		queue!(self.output, MoveRight(1))?;

		Ok(())
	}

	pub fn backspace(&mut self) -> TerminalResult<()> {
		if self.current_column == 0 && self.current_row == 0 {
			return Ok(());
		}

		// Move the contents of this line to the previous.
		if self.current_column == 0 {
			let previous_line_len = self.buffer[self.current_row - 1].len();

			let current_line = self.buffer.remove(self.current_row);

			self.buffer[self.current_row - 1].extend(current_line);

			// Clears buffer so text can be rerendered cleanly.
			queue!(self.output, Clear(ClearType::FromCursorDown))?;

			// Move curser up to previous line before concatenating the lines.
			queue!(
				self.output,
				MoveUp(1),
				MoveToColumn(previous_line_len as u16)
			)?;

			self.current_row -= 1;
			self.current_column = previous_line_len;

			return Ok(());
		}

		self.buffer[self.current_row].remove(self.current_column - 1);

		self.current_column -= 1;

		queue!(self.output, MoveLeft(1))?;

		Ok(())
	}

	pub fn enter(&mut self) -> TerminalResult<()> {
		// The characters to move to the next line.
		let next_line = self.buffer[self.current_row].split_off(self.current_column);

		self.buffer.insert(self.current_row + 1, next_line);
		self.current_row += 1;
		self.current_column = 0;

		queue!(self.output, MoveToNextLine(1))?;

		Ok(())
	}

	pub fn move_cursor(&mut self, direction: Direction) -> TerminalResult<()> {
		match direction {
			Direction::Up => self.move_up()?,
			Direction::Down => self.move_down()?,
			Direction::Left => self.move_left()?,
			Direction::Right => self.move_right()?,
		}

		Ok(())
	}

	fn move_right(&mut self) -> TerminalResult<()> {
		// At end of final line.
		if self.current_row + 1 == self.buffer.len() {
			return Ok(());
		}

		if self.current_column < self.buffer[self.current_row].len() {
			// Not at end of line.

			queue!(self.output, MoveRight(1))?;
		} else {
			queue!(self.output, MoveToNextLine(1))?;
			self.current_column = 0;
		}

		self.current_column += 1;

		Ok(())
	}

	fn move_left(&mut self) -> TerminalResult<()> {
		// At start top left corner.
		if self.current_column == 0 && self.current_row == 0 {
			return Ok(());
		}

		if self.current_column > 0 {
			// Not at start of line.

			queue!(self.output, MoveLeft(1))?;
		} else {
			// To move the curser to the end of the line.
			let previous_line_length = self.buffer[self.current_row - 1].len();
			queue!(
				self.output,
				MoveTo(previous_line_length as u16, self.current_row as u16 - 1)
			)?;
			self.current_column = previous_line_length;
		}

		self.current_column -= 1;

		Ok(())
	}

	// TODO: make this and move_up keep track of the column so the curser
	// can go back after going on from a shorter line.... if that makes sense
	fn move_down(&mut self) -> TerminalResult<()> {
		if self.current_row >= self.buffer.len() - 1 {
			return Ok(());
		}

		let next_row_length = self.buffer[self.current_row + 1].len();

		if self.current_column > next_row_length {
			queue!(self.output, MoveToColumn(next_row_length as u16))?;
			self.current_column = next_row_length;
		}

		self.current_row += 1;
		queue!(self.output, MoveDown(1))?;

		Ok(())
	}

	fn move_up(&mut self) -> TerminalResult<()> {
		if self.current_row == 0 {
			return Ok(());
		}

		let previous_row_length = self.buffer[self.current_row - 1].len();

		if self.current_column > previous_row_length {
			queue!(self.output, MoveToColumn(previous_row_length as u16))?;
			self.current_column = previous_row_length;
		}

		self.current_row -= 1;
		queue!(self.output, MoveUp(1))?;

		Ok(())
	}

	/// Resize the editor window on terminal resize event.
	/// TODO: handle what happens to when columns resizes to less that `current_column`
	pub fn resize(&mut self, columns: u16, rows: u16) -> TerminalResult<()> {
		self.columns = columns;
		self.rows = rows;

		queue!(self.output, Clear(ClearType::All))?;

		self.print_buffer()?;

		Ok(())
	}

	pub fn print_buffer(&mut self) -> TerminalResult<()> {
		queue!(self.output, SavePosition, MoveTo(0, 0))?;

		for line in &self.buffer {
			queue!(
				self.output,
				Clear(ClearType::CurrentLine),
				Print(line.iter().collect::<String>()),
				MoveToNextLine(1)
			)?;
		}

		self.queue_status()?;

		queue!(self.output, RestorePosition)?;

		self.output.flush()?;

		Ok(())
	}

	/// Queues the status bar to to the terminals output.
	///
	/// Must be followed by an execute otherwise nothing will happen.
	fn queue_status(&mut self) -> TerminalResult<()> {
		let left = format!(" {} ", self.mode);

		let right = format!(" {}:{}", self.current_row, self.current_column,);

		queue!(
			self.output,
			MoveToRow(self.rows - 2),
			SetBackgroundColor(crossterm::style::Color::DarkGrey),
			Clear(ClearType::CurrentLine),
			Print(left),
			MoveToColumn(self.columns - right.len() as u16),
			Print(right),
			ResetColor,
		)?;

		Ok(())
	}
}
