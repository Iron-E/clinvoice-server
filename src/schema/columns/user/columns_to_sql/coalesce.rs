use core::fmt::{Display, Formatter, Result};
pub struct Coalesce<Left, Right>(pub Left, pub Right)
where
	Left: Display,
	Right: Display;

impl<Left, Right> Display for Coalesce<Left, Right>
where
	Left: Display,
	Right: Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result
	{
		write!(f, "COALESCE(NULLIF({}, ''), {})", self.0, self.1)
	}
}
