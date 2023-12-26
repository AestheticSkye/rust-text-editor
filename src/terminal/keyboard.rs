//! Terminal methods to handle keyboard interaction of the terminal.
mod cursor;

use crossterm::cursor::{MoveLeft, MoveRight, MoveToColumn, MoveToNextLine, MoveUp};
use crossterm::event::KeyCode;
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType};

use crate::direction::Direction;
use crate::mode::Mode;
use crate::terminal::Terminal;
use crate::TerminalResult;

#[allow(clippy::cast_possible_truncation)]
impl Terminal {
	pub(super) fn move_cursor(&mut self, direction: Direction) -> TerminalResult<()> {
		match direction {
			Direction::Up => self.move_up()?,
			Direction::Down => self.move_down()?,
			Direction::Left => self.move_left()?,
			Direction::Right => self.move_right()?,
		}

		Ok(())
	}

	#[allow(clippy::match_same_arms)]
	pub(super) fn insert_mode_key_event(&mut self, keycode: KeyCode) -> TerminalResult<()> {
		match keycode {
			KeyCode::Backspace => self.backspace()?,
			KeyCode::Enter => self.enter()?,
			KeyCode::Home => {}
			KeyCode::End => {}
			KeyCode::PageUp => {}
			KeyCode::PageDown => {}
			KeyCode::Tab => self.insert_char('\t')?,
			KeyCode::BackTab => {}
			KeyCode::Delete => self.delete()?,
			KeyCode::Insert => {}
			KeyCode::Char(char) => self.insert_char(char)?,
			KeyCode::Esc => self.mode = Mode::Normal,
			KeyCode::KeypadBegin => {}
			KeyCode::Modifier(_) => {}
			_ => {}
		};

		Ok(())
	}

	// TODO: make this respect `\t`.
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

			// Clears buffer so text can be re-rendered cleanly.
			queue!(self.output, Clear(ClearType::FromCursorDown))?;

			// Move cursor up to previous line before concatenating the lines.
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

	pub fn delete(&mut self) -> TerminalResult<()> {
		todo!()
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
}
