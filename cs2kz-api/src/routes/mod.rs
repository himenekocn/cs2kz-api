use std::fmt::{self, Display};

use serde::{Deserialize, Deserializer};
use sqlx::{MySql, QueryBuilder};

pub mod health;
pub mod players;
pub mod bans;
pub mod maps;
pub mod servers;
pub mod records;
pub mod auth;

/// A filter to use in database queries.
///
/// Can be [`.push()`]'ed to a query to concatenate filters. After pushing, you can call
/// [`.switch()`] so the next push will use [`Filter::And`].
///
/// [`.push()`]: sqlx::QueryBuilder::push
/// [`.switch()`]: Self::switch
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
	#[default]
	Where,

	And,
}

impl Filter {
	pub const fn new() -> Self {
		Self::Where
	}

	/// Switch `self` to [`Filter::And`].
	pub fn switch(&mut self) {
		*self = Self::And;
	}
}

impl Display for Filter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Filter::Where => " WHERE ",
			Filter::And => " AND ",
		})
	}
}

/// A utility type for deserializing a [`u64`].
///
/// * `DEFAULT`: the fallback value to be used if the actual value was null (defaults to 0)
/// * `MAX`: the maximum value that is allowed (defaults to [`u64::MAX`])
/// * `MIN`: the minimum value that is allowed (defaults to 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedU64<const DEFAULT: u64 = 0, const MAX: u64 = { u64::MAX }, const MIN: u64 = 0> {
	pub value: u64,
}

impl<'de, const DEFAULT: u64, const MAX: u64, const MIN: u64> Deserialize<'de>
	for BoundedU64<DEFAULT, MAX, MIN>
{
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		use serde::de::Error as E;

		let value = match Option::<u64>::deserialize(deserializer)? {
			None => DEFAULT,
			// No `Some(value @ MIN..=MAX)` pattern matching :(
			Some(value) if (MIN..=MAX).contains(&value) => value,
			Some(out_of_bounds) => {
				return Err(E::custom(format!(
					"expected integer in the range of {MIN}..={MAX} but got {out_of_bounds}"
				)));
			}
		};

		Ok(Self { value })
	}
}

pub fn push_limit<const LIMIT_LIMIT: u64>(
	query: &mut QueryBuilder<'_, MySql>,
	offset: BoundedU64,
	limit: BoundedU64<100, LIMIT_LIMIT>,
) {
	query
		.push(" LIMIT ")
		.push_bind(offset.value)
		.push(",")
		.push_bind(limit.value);
}
