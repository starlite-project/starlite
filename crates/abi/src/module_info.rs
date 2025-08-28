use std::{ffi::CStr, os::raw::c_char, slice, str};

use serde::{Serialize, Serializer, ser::SerializeStruct as _};

use super::{FunctionDefinition, TypeDefinition};

#[repr(C)]
pub struct ModuleInfo<'a> {
	pub(crate) path: *const c_char,
	pub(crate) functions: *const FunctionDefinition<'a>,
	pub(crate) types: *const TypeDefinition<'a>,
	pub num_functions: u32,
	pub num_types: u32,
}

impl<'a> ModuleInfo<'a> {
	#[must_use]
	pub const fn path(&self) -> &str {
		unsafe { str::from_utf8_unchecked(CStr::from_ptr(self.path).to_bytes()) }
	}

	#[must_use]
	pub const fn functions(&self) -> &[FunctionDefinition<'a>] {
		if matches!(self.num_functions, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.functions, self.num_functions as usize) }
		}
	}

	#[must_use]
	pub const fn types(&self) -> &[TypeDefinition<'a>] {
		if matches!(self.num_types, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.types, self.num_types as usize) }
		}
	}
}

impl Serialize for ModuleInfo<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = serializer.serialize_struct("ModuleInfo", 3)?;

		s.serialize_field("path", self.path())?;
		s.serialize_field("functions", self.functions())?;
		s.serialize_field("types", self.types())?;
		s.end()
	}
}

unsafe impl Send for ModuleInfo<'_> {}
unsafe impl Sync for ModuleInfo<'_> {}
