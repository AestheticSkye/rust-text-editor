use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
	Normal,
	Insert,
}

impl Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Mode::Normal => "NORMAL",
				Mode::Insert => "INSERT",
			}
			.to_string()
		)
	}
}
