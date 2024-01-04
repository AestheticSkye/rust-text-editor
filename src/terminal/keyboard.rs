//! Terminal methods to handle keyboard interaction of the terminal.

mod cursor;

use crossterm::cursor::{MoveRight, MoveToColumn};
use crossterm::event::KeyCode;
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType};

use crate::direction::Direction;
use crate::mode::Mode;
use crate::terminal::Terminal;
use crate::{clipboard, Result};

#[allow(clippy::cast_possible_truncation)]
impl Terminal {
	pub(super) fn move_cursor(&mut self, direction: Direction) -> Result<()> {
		match direction {
			Direction::Up => self.move_up()?,
			Direction::Down => self.move_down()?,
			Direction::Left => self.move_left()?,
			Direction::Right => self.move_right()?,
		}

		Ok(())
	}

	#[allow(clippy::match_same_arms)]
	pub(super) fn insert_mode_key_event(&mut self, keycode: KeyCode) -> Result<()> {
		match keycode {
			KeyCode::Backspace => self.backspace()?,
			KeyCode::Enter => self.enter()?,
			KeyCode::Home => {}
			KeyCode::End => {}
			KeyCode::PageUp => {}
			KeyCode::PageDown => {}
			KeyCode::Tab => self.insert_chars(&['\t'])?,
			KeyCode::BackTab => {}
			KeyCode::Delete => self.delete()?,
			KeyCode::Insert => {}
			KeyCode::Char(char) => self.insert_chars(&[char])?,
			KeyCode::Esc => self.mode = Mode::Normal,
			KeyCode::KeypadBegin => {}
			KeyCode::Modifier(_) => {}
			_ => {}
		};

		Ok(())
	}

	#[allow(clippy::unnecessary_wraps)]
	pub(super) fn normal_mode_key_event(&mut self, keycode: KeyCode) -> Result<()> {
		match keycode {
			KeyCode::Char(char) if self.mode == Mode::Normal => match char {
				'q' => self.is_running = false,
				'i' => self.mode = Mode::Insert,
				'p' => self.paste()?,
				_ => {}
			},
			_ => {}
		}
		Ok(())
	}

	fn paste(&mut self) -> Result<()> {
		let Some(text) = clipboard::get_text()? else {
			return Ok(());
		};

		self.insert_chars(&text.chars().collect::<Vec<char>>())?;

		Ok(())
	}

	/// Insert characters into the editors buffer on the current line.
	fn insert_chars(&mut self, chars: &[char]) -> Result<()> {
		for char in chars {
			if *char == '\n' {
				self.enter()?;
			} else {
				self.buffer[self.current_row].insert(self.current_column, *char);
				self.current_column += 1;
			}
		}

		queue!(self.output, MoveRight(chars.len() as u16))?;

		Ok(())
	}

	fn backspace(&mut self) -> Result<()> {
		if self.current_column == 0 && self.current_row == 0 {
			return Ok(());
		}

		// If cursor is not at the start of the line, just remove the character and move.
		if self.current_column != 0 {
			self.buffer[self.current_row].remove(self.current_column - 1);

			self.move_left()?;

			return Ok(());
		}

		let previous_line_len = self.buffer[self.current_row - 1].len();
		let current_line = self.buffer.remove(self.current_row);

		// Move the contents of this line to the previous.
		self.buffer[self.current_row - 1].extend(current_line);

		// Clears buffer so text can be re-rendered cleanly.
		queue!(self.output, Clear(ClearType::FromCursorDown))?;

		self.move_left()?;

		self.current_column = previous_line_len;
		queue!(self.output, MoveToColumn(previous_line_len as u16))?;

		Ok(())
	}

	fn delete(&mut self) -> Result<()> {
		todo!()
	}

	fn enter(&mut self) -> Result<()> {
		// The characters to move to the next line.
		let next_line = self.buffer[self.current_row].split_off(self.current_column);

		self.buffer.insert(self.current_row + 1, next_line);

		self.move_right()?;
		Ok(())
	}
}
