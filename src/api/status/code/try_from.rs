//! Implementations of [`TryFrom`] for [`Code`]

use core::num::TryFromIntError;

use super::Code;

impl TryFrom<u8> for Code
{
	type Error = TryFromIntError;

	fn try_from(value: u8) -> Result<Self, Self::Error>
	{
		Ok(match value
		{
			v if v == Self::ApiVersionHeaderMissing as u8 => Self::ApiVersionHeaderMissing,
			v if v == Self::ApiVersionMismatch as u8 => Self::ApiVersionMismatch,
			v if v == Self::BadArguments as u8 => Self::BadArguments,
			v if v == Self::CryptError as u8 => Self::CryptError,
			v if v == Self::Database as u8 => Self::Database,
			v if v == Self::EncodingError as u8 => Self::EncodingError,
			v if v == Self::InvalidCredentials as u8 => Self::InvalidCredentials,
			v if v == Self::LoginError as u8 => Self::LoginError,
			v if v == Self::Other as u8 => Self::Other,
			v if v == Self::PasswordExpired as u8 => Self::PasswordExpired,
			v if v == Self::PermissionsError as u8 => Self::PermissionsError,
			v if v == Self::SqlError as u8 => Self::SqlError,
			v if v == Self::Success as u8 => Self::Success,
			v if v == Self::SuccessForPermissions as u8 => Self::SuccessForPermissions,
			v if v == Self::Unauthorized as u8 => Self::Unauthorized,

			// HACK: `TryFromIntError` has a private constructor… why?
			_ => return Err(<u8 as TryFrom<u16>>::try_from(300).unwrap_err()),
		})
	}
}
