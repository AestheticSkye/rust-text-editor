use crossterm::event::{read, Event, KeyCode};

use crate::direction::Direction;
use crate::mode::Mode;
use crate::terminal::Terminal;
use crate::TerminalResult;

pub fn handle_event(terminal: &mut Terminal) -> TerminalResult<bool> {
	match read()? {
		Event::Key(key_event) => match key_event.code {
			KeyCode::Left => terminal.move_cursor(Direction::Left)?,
			KeyCode::Right => terminal.move_cursor(Direction::Right)?,
			KeyCode::Up => terminal.move_cursor(Direction::Up)?,
			KeyCode::Down => terminal.move_cursor(Direction::Down)?,
			_ => match terminal.mode {
				Mode::Insert => insert_mode_key_event(key_event.code, terminal)?,
				Mode::Normal => match key_event.code {
					KeyCode::Char(char) if terminal.mode == Mode::Normal => match char {
						'q' => return Ok(true),
						'i' => terminal.mode = Mode::Insert,
						_ => {}
					},
					_ => {}
				},
			},
		},
		Event::Resize(columns, rows) => terminal.resize(columns, rows)?,
		_ => {}
	}

	Ok(false)
}

#[allow(clippy::match_same_arms)]
pub fn insert_mode_key_event(keycode: KeyCode, terminal: &mut Terminal) -> TerminalResult<()> {
	match keycode {
		KeyCode::Backspace => terminal.backspace()?,
		KeyCode::Enter => terminal.enter()?,
		KeyCode::Home => {}
		KeyCode::End => {}
		KeyCode::PageUp => {}
		KeyCode::PageDown => {}
		KeyCode::Tab => {}
		KeyCode::BackTab => {}
		KeyCode::Delete => {}
		KeyCode::Insert => {}
		KeyCode::F(_) => {}
		KeyCode::Char(char) => terminal.insert_char(char)?,
		KeyCode::Null => {}
		KeyCode::Esc => terminal.mode = Mode::Normal,
		KeyCode::CapsLock => {}
		KeyCode::ScrollLock => {}
		KeyCode::NumLock => {}
		KeyCode::PrintScreen => {}
		KeyCode::Pause => {}
		KeyCode::Menu => {}
		KeyCode::KeypadBegin => {}
		KeyCode::Media(_) => {}
		KeyCode::Modifier(_) => {}
		_ => {}
	};

	Ok(())
}
