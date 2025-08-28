use std::{
	ffi::CStr,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	os::raw::c_char,
	str,
};

use serde::{Serialize, Serializer, ser::SerializeStruct as _};

use crate::{Guid, StructDefinition, TypeId};

#[repr(C)]
pub struct TypeDefinition<'a> {
	pub name: *const c_char,
	pub(crate) size: u32,
	pub(crate) alignment: u8,
	pub data: TypeDefinitionData<'a>,
}

impl<'a> TypeDefinition<'a> {
	#[must_use]
	pub fn is_instance_of(&self, type_id: &TypeId<'a>) -> bool {
		match (&self.data, type_id) {
			(TypeDefinitionData::Struct(s), TypeId::Concrete(guid)) => &s.guid == guid,
			_ => false,
		}
	}

	#[must_use]
	pub const fn name(&self) -> &str {
		unsafe { str::from_utf8_unchecked(CStr::from_ptr(self.name).to_bytes()) }
	}

	#[must_use]
	pub const fn as_concrete(&self) -> &Guid {
		match &self.data {
			TypeDefinitionData::Struct(s) => &s.guid,
		}
	}

	#[must_use]
	pub const fn as_struct(&self) -> Option<&StructDefinition<'a>> {
		let TypeDefinitionData::Struct(s) = &self.data;
		Some(s)
	}

	#[must_use]
	pub const fn size(&self) -> usize {
		self.size as usize
	}

	#[must_use]
	pub const fn size_in_bytes(&self) -> usize {
		self.size().div_ceil(8)
	}

	#[must_use]
	pub const fn alignment(&self) -> usize {
		self.alignment as usize
	}
}

impl Debug for TypeDefinition<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("TypeDefinition")
			.field("name", &self.name())
			.field("size", &self.size)
			.field("alignment", &self.alignment)
			.field("data", &self.data)
			.finish()
	}
}

impl Display for TypeDefinition<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(self.name())
	}
}

impl Eq for TypeDefinition<'_> {}

impl PartialEq for TypeDefinition<'_> {
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&self.size, &other.size)
			&& PartialEq::eq(&self.alignment, &other.alignment)
			&& PartialEq::eq(&self.data, &other.data)
	}
}

impl Serialize for TypeDefinition<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = serializer.serialize_struct("TypeDefinition", 4)?;
		s.serialize_field("name", self.name())?;
		s.serialize_field("size", &self.size)?;
		s.serialize_field("alignment", &self.alignment)?;
		s.serialize_field("data", &self.data)?;
		s.end()
	}
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for TypeDefinition<'_> {}
unsafe impl Sync for TypeDefinition<'_> {}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum TypeDefinitionData<'a> {
	Struct(StructDefinition<'a>),
}

impl TypeDefinitionData<'_> {
	#[must_use]
	pub const fn is_struct(&self) -> bool {
		matches!(self, Self::Struct(..))
	}
}

pub trait HasStaticTypeName {
	fn type_name() -> &'static CStr;
}
