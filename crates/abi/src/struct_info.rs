use std::{ffi::CStr, os::raw::c_char, slice, str};

use serde::{Serialize, Serializer, ser::SerializeStruct as _};
use serde_repr::Serialize_repr;

use crate::{Guid, TypeId};

#[repr(C)]
#[derive(Debug)]
pub struct StructDefinition<'a> {
	pub guid: Guid,
	pub field_names: *const *const c_char,
	pub(crate) field_types: *const TypeId<'a>,
	pub(crate) field_offsets: *const u16,
	pub(crate) num_fields: u16,
	pub memory_type: StructMemoryType,
}

impl<'a> StructDefinition<'a> {
	pub fn field_names(&self) -> impl Iterator<Item = &str> {
		let field_names = if matches!(self.num_fields, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.field_names, self.num_fields()) }
		};

		field_names
			.iter()
			.map(|n| unsafe { str::from_utf8_unchecked(CStr::from_ptr(*n).to_bytes()) })
	}

	#[must_use]
	pub const fn field_types(&self) -> &[TypeId<'a>] {
		if matches!(self.num_fields, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.field_types, self.num_fields()) }
		}
	}

	#[must_use]
	pub const fn field_offsets(&self) -> &[u16] {
		if matches!(self.num_fields, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.field_offsets, self.num_fields()) }
		}
	}

	#[must_use]
	pub const fn num_fields(&self) -> usize {
		self.num_fields as usize
	}
}

impl Eq for StructDefinition<'_> {}

impl PartialEq for StructDefinition<'_> {
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&self.guid, &other.guid)
	}
}

impl Serialize for StructDefinition<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		struct Field<'a> {
			name: &'a str,
			r#type: &'a TypeId<'a>,
			offset: &'a u16,
		}

		let mut s = serializer.serialize_struct("StructInfo", 3)?;

		s.serialize_field("guid", &self.guid)?;
		s.serialize_field(
			"fields",
			&self
				.field_names()
				.zip(self.field_types())
				.zip(self.field_offsets())
				.map(|((name, ty), offset)| Field {
					name,
					r#type: ty,
					offset,
				})
				.collect::<Vec<_>>(),
		)?;
		s.serialize_field("memory_type", &self.memory_type)?;

		s.end()
	}
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize_repr)]
pub enum StructMemoryType {
	#[default]
	Gc,
	Value,
}
