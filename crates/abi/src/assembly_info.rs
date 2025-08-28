use std::{ffi::CStr, os::raw::c_char, slice, str};

use serde::{Serialize, Serializer, ser::SerializeStruct as _};

use super::{DispatchTable, ModuleInfo, TypeLut};

#[repr(C)]
pub struct AssemblyInfo<'a> {
	pub symbols: ModuleInfo<'a>,
	pub dispatch_table: DispatchTable<'a>,
	pub type_lut: TypeLut<'a>,
	pub(crate) dependencies: *const *const c_char,
	pub num_dependencies: u32,
}

impl AssemblyInfo<'_> {
	pub fn dependencies(&self) -> impl Iterator<Item = &str> {
		let deps = if matches!(self.num_dependencies, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.dependencies, self.num_dependencies as usize) }
		};

		deps.iter()
			.map(|d| unsafe { str::from_utf8_unchecked(CStr::from_ptr(*d).to_bytes()) })
	}
}

impl Serialize for AssemblyInfo<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = serializer.serialize_struct("AssemblyInfo", 4)?;
		s.serialize_field("symbols", &self.symbols)?;
		s.serialize_field("dispatch_table", &self.dispatch_table)?;
		s.serialize_field("type_lut", &self.type_lut)?;
		s.serialize_field("dependencies", &self.dependencies().collect::<Vec<_>>())?;
		s.end()
	}
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for AssemblyInfo<'_> {}
unsafe impl Sync for AssemblyInfo<'_> {}

#[cfg(test)]
mod tests {
	use std::ffi::CString;

	use crate::test_utils::{
		FAKE_DEPENDENCY, FAKE_MODULE_PATH, fake_assembly_info, fake_dispatch_table,
		fake_module_info, fake_type_lut,
	};

	#[test]
	fn assembly_info_dependencies() {
		let module_path = CString::new(FAKE_MODULE_PATH).expect("invalid fake module path");
		let module = fake_module_info(&module_path, &[], &[]);

		let dispatch_table = fake_dispatch_table(&[], &mut []);
		let type_lut = fake_type_lut(&[], &mut [], &[]);

		let dependency = CString::new(FAKE_DEPENDENCY).expect("invalid fake dependency");
		let dependencies = &[dependency.as_ptr()];
		let assembly = fake_assembly_info(module, dispatch_table, type_lut, dependencies);

		assert_eq!(assembly.dependencies().count(), dependencies.len());
		for (lhs, rhs) in assembly.dependencies().zip(std::iter::once(&FAKE_DEPENDENCY)) {
			assert_eq!(lhs, *rhs);
		}
	}
}
