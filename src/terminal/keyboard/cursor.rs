//! Terminal methods to handle the movement of the terminals cursor.
use crossterm::cursor::{
	MoveDown, MoveLeft, MoveRight, MoveToColumn, MoveToNextLine, MoveToRow, MoveUp,
};
use crossterm::queue;

use crate::terminal::Terminal;
use crate::TerminalResult;

#[allow(clippy::cast_possible_truncation)]
impl Terminal {
	pub(super) fn move_right(&mut self) -> TerminalResult<()> {
		if self.current_column < self.buffer[self.current_row].len() {
			// Not at end of line.

			queue!(self.output, MoveRight(1))?;
			self.current_column += 1;
		} else {
			// At end of final line.
			if self.current_row + 1 == self.buffer.len() {
				return Ok(());
			}

			queue!(self.output, MoveToNextLine(1))?;
			self.current_column = 0;
			self.current_row += 1;
		}

		Ok(())
	}

	pub(super) fn move_left(&mut self) -> TerminalResult<()> {
		// At start top left corner.
		if self.current_column == 0 && self.current_row == 0 {
			return Ok(());
		}

		if self.current_column > 0 {
			// Not at start of line.

			queue!(self.output, MoveLeft(1))?;
			self.current_column -= 1;
		} else {
			// To move the cursor to the end of the line.
			let previous_line_length = self.buffer[self.current_row - 1].len();
			queue!(
				self.output,
				MoveToColumn(previous_line_length as u16),
				MoveToRow(self.current_row as u16 - 1)
			)?;
			self.current_column = previous_line_length;
			self.current_row -= 1;
		}

		Ok(())
	}

	// TODO: make this and move_up keep track of the column so the cursor
	// can go back after going on from a shorter line.... if that makes sense
	pub(super) fn move_down(&mut self) -> TerminalResult<()> {
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

	pub(super) fn move_up(&mut self) -> TerminalResult<()> {
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
}
