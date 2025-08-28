use std::{
	ffi::{CStr, c_void},
	os::raw::c_char,
	slice, str,
};

use serde::{Serialize, Serializer, ser::SerializeStruct as _};

use super::{HasStaticTypeId, TypeId};

#[repr(C)]
#[derive(Clone)]
pub struct FunctionDefinition<'a> {
	pub prototype: FunctionPrototype<'a>,
	pub fn_ptr: *const c_void,
}

impl Serialize for FunctionDefinition<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = serializer.serialize_struct("FunctionDefinition", 1)?;
		s.serialize_field("prototype", &self.prototype)?;
		s.skip_field("fn_ptr")?;
		s.end()
	}
}

unsafe impl Send for FunctionDefinition<'_> {}
unsafe impl Sync for FunctionDefinition<'_> {}

#[repr(C)]
#[derive(Clone)]
pub struct FunctionPrototype<'a> {
	pub name: *const c_char,
	pub signature: FunctionSignature<'a>,
}

impl FunctionPrototype<'_> {
	#[must_use]
	pub const fn name(&self) -> &str {
		unsafe { str::from_utf8_unchecked(CStr::from_ptr(self.name).to_bytes()) }
	}
}

impl Serialize for FunctionPrototype<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = serializer.serialize_struct("FunctionPrototype", 2)?;

		s.serialize_field("name", self.name())?;
		s.serialize_field("signature", &self.signature)?;
		s.end()
	}
}

unsafe impl Send for FunctionPrototype<'_> {}
unsafe impl Sync for FunctionPrototype<'_> {}

#[repr(C)]
#[derive(Clone)]
pub struct FunctionSignature<'a> {
	pub arg_types: *const TypeId<'a>,
	pub return_type: TypeId<'a>,
	pub num_arg_types: u16,
}

impl<'a> FunctionSignature<'a> {
	#[must_use]
	pub const fn arg_types(&self) -> &[TypeId<'a>] {
		if matches!(self.num_arg_types, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.arg_types, self.num_arg_types as usize) }
		}
	}

	#[must_use]
	pub fn return_type(&self) -> Option<TypeId<'a>> {
		if <()>::type_id() == &self.return_type {
			None
		} else {
			Some(self.return_type.clone())
		}
	}
}

impl Serialize for FunctionSignature<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = serializer.serialize_struct("FunctionSignature", 2)?;

		s.serialize_field("arg_types", self.arg_types())?;
		s.serialize_field("return_type", &self.return_type())?;
		s.end()
	}
}

unsafe impl Send for FunctionSignature<'_> {}
unsafe impl Sync for FunctionSignature<'_> {}
