use winvoice_schema::chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc};

/// Create a [`DateTime<Utc>`] out of some [`Local`] [`NaiveDateTime`].
pub(crate) fn naive_local_datetime_to_utc(d: NaiveDateTime) -> DateTime<Utc>
{
	Local.ymd(d.year(), d.month(), d.day()).and_hms(d.hour(), d.minute(), d.second()).into()
}

/// A temporary directory which can be used to write files into for `test`.
#[cfg(test)]
pub(crate) fn temp_dir(test: &str) -> std::path::PathBuf
{
	let mut parent = std::env::temp_dir();
	parent.push("winvoice-server");
	parent.push(test);

	std::fs::create_dir_all(&parent).unwrap();

	parent
}
