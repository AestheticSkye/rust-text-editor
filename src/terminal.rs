//! The terminal used for the editor.
//!
//! This module is seperated into multiple submodules, with input handling within `self::keyboard`
//! as well as cursor management within `self::keyboard::cursor`

mod keyboard;

use std::io::{stdout, Write};

use crossterm::cursor::{
	MoveTo, MoveToColumn, MoveToNextLine, MoveToRow, RestorePosition, SavePosition, Show,
};
use crossterm::event::{read, Event, KeyCode};
use crossterm::style::{Print, ResetColor, SetBackgroundColor};
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
	LeaveAlternateScreen,
};
use crossterm::{execute, queue};

use crate::direction::Direction;
use crate::mode::Mode;
use crate::Result;

/// Terminal system for running the editor.
pub struct Terminal {
	/// Terminals width, changed on resize event.
	columns: u16,
	/// Terminals height, changed on resize event.
	rows: u16,
	/// Current cursor column location.
	current_column: usize,
	/// Current cursor row location.
	current_row: usize,
	/// Test buffer of user input.
	buffer: Vec<Vec<char>>,
	/// The writer for the terminal, generally stdio.
	output: Box<dyn Write>,
	/// The terminals current running state.
	is_running: bool,
	/// The current mode for terminal interaction.
	mode: Mode,
}

impl Drop for Terminal {
	fn drop(&mut self) {
		#[allow(clippy::expect_used)]
		disable_raw_mode().expect("Failed to disable raw mode");
		execute!(self.output, ResetColor, Show, LeaveAlternateScreen)
			.expect("Failed to clean up screen");
	}
}

#[allow(clippy::cast_possible_truncation)]
impl Terminal {
	pub fn new() -> Result<Self> {
		execute!(stdout(), EnterAlternateScreen)?;
		enable_raw_mode()?;

		// Register a panic hook to run the drop behavior if the program panics.
		// Required because terminal is dropped *after* it attempts to print the panic message.
		// Which causes the panic message to not appear in terminal.
		std::panic::set_hook(Box::new(|panic_info| {
			#[allow(clippy::expect_used)]
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
			is_running: true,
			output: Box::new(stdout()),
		})
	}

	pub const fn is_running(&self) -> bool {
		self.is_running
	}

	/// Handle an event in the terminals event loop.
	///
	/// Returns true if functions signifies a exit command.
	pub fn handle_event(&mut self) -> Result<()> {
		match read()? {
			Event::Key(key_event) => match key_event.code {
				// Movement works in any mode.
				KeyCode::Left => self.move_cursor(Direction::Left)?,
				KeyCode::Right => self.move_cursor(Direction::Right)?,
				KeyCode::Up => self.move_cursor(Direction::Up)?,
				KeyCode::Down => self.move_cursor(Direction::Down)?,
				// Mode specific inputs.
				_ => match self.mode {
					Mode::Insert => self.insert_mode_key_event(key_event.code)?,
					Mode::Normal => self.normal_mode_key_event(key_event.code)?,
				},
			},
			Event::Resize(columns, rows) => self.resize(columns, rows)?,
			_ => {}
		}

		Ok(())
	}

	/// Resize the editor window on terminal resize event.
	/// TODO: handle what happens to when columns resizes to less that `current_column`
	pub fn resize(&mut self, columns: u16, rows: u16) -> Result<()> {
		self.columns = columns;
		self.rows = rows;

		queue!(self.output, Clear(ClearType::All))?;

		self.print_buffer()?;

		Ok(())
	}

	pub fn print_buffer(&mut self) -> Result<()> {
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
	fn queue_status(&mut self) -> Result<()> {
		let left = format!(" {} ", self.mode);
		let right = format!(" {}:{} ", self.current_row, self.current_column);

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
