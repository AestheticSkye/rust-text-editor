use std::io::{stdout, Write};

use crossterm::cursor::{
	MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveToNextLine, MoveToRow, RestorePosition,
	SavePosition, Show,
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

impl Terminal {
	pub fn new() -> TerminalResult<Terminal> {
		execute!(stdout(), EnterAlternateScreen)?;
		enable_raw_mode()?;

		// Register a panic hook to run the drop behavior if the program panics.
		// Required as panic errors will not get printed otherwise.
		std::panic::set_hook(Box::new(|panic_info| {
			disable_raw_mode().expect("Failed to disable raw mode");
			execute!(stdout(), ResetColor, Show, LeaveAlternateScreen)
				.expect("Failed to clean up screen");
			println!("{panic_info}")
		}));

		let (columns, rows) = size()?;

		let terminal = Self {
			buffer: vec![vec![]],
			columns,
			rows,
			mode: Mode::Normal,
			current_column: 0,
			current_row: 0,
			output: Box::new(stdout()),
		};

		Ok(terminal)
	}

	pub fn insert_char(&mut self, char: char) -> TerminalResult<()> {
		self.buffer[self.current_row].insert(self.current_column.into(), char);

		self.current_column += 1;

		queue!(self.output, MoveRight(1))?;

		Ok(())
	}

	// TODO: Make this respect backspacing newlines
	pub fn backspace(&mut self) -> TerminalResult<()> {
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
			Direction::Up => todo!(),
			Direction::Down => todo!(),
			Direction::Left => {
				if self.current_column > 0 {
					queue!(self.output, MoveLeft(1))?;
					self.current_column -= 1;
				}
			}
			Direction::Right => {
				if (self.current_column) < self.buffer[self.current_row].len() {
					queue!(self.output, MoveRight(1))?;
					self.current_column += 1;
				}
			}
		}

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
		let left = format!(" {} ", self.mode.to_string());

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
