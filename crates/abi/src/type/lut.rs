use std::{
	ffi::{self, CStr},
	os::raw::c_char,
	slice, str,
};

use itertools::izip;
use serde::{Serialize, Serializer, ser::SerializeSeq as _};

use super::TypeId;

#[repr(C)]
pub struct TypeLut<'a> {
	pub(crate) type_ids: *const TypeId<'a>,
	pub(crate) type_handles: *mut *const ffi::c_void,
	pub(crate) type_names: *const *const c_char,
	pub num_entries: u32,
}

#[allow(clippy::iter_on_empty_collections)]
impl<'a> TypeLut<'a> {
	pub fn iter(&self) -> impl Iterator<Item = (&TypeId<'_>, &*const ffi::c_void, &str)> {
		let (type_ids, type_ptrs, type_names) = if matches!(self.num_entries, 0) {
			(([]).iter(), ([]).iter(), ([]).iter())
		} else {
			let ptrs =
				unsafe { slice::from_raw_parts_mut(self.type_handles, self.num_entries as usize) };
			let type_ids =
				unsafe { slice::from_raw_parts(self.type_ids, self.num_entries as usize) };
			let type_names =
				unsafe { slice::from_raw_parts(self.type_names, self.num_entries as usize) };

			(type_ids.iter(), ptrs.iter(), type_names.iter())
		};

		izip!(type_ids, type_ptrs, type_names).map(|(id, ptr, type_name)| {
			(id, ptr, unsafe {
				std::str::from_utf8_unchecked(CStr::from_ptr(*type_name).to_bytes())
			})
		})
	}

	pub fn iter_mut(
		&mut self,
	) -> impl Iterator<Item = (&TypeId<'_>, &mut *const ffi::c_void, &str)> {
		let (type_ids, type_ptrs, type_names) = if matches!(self.num_entries, 0) {
			(([]).iter(), ([]).iter_mut(), ([]).iter())
		} else {
			let ptrs =
				unsafe { slice::from_raw_parts_mut(self.type_handles, self.num_entries as usize) };
			let type_ids =
				unsafe { slice::from_raw_parts(self.type_ids, self.num_entries as usize) };
			let type_names =
				unsafe { slice::from_raw_parts(self.type_names, self.num_entries as usize) };

			(type_ids.iter(), ptrs.iter_mut(), type_names.iter())
		};

		izip!(type_ids, type_ptrs, type_names).map(|(id, ptr, type_name)| {
			(id, ptr, unsafe {
				std::str::from_utf8_unchecked(CStr::from_ptr(*type_name).to_bytes())
			})
		})
	}

	pub const fn type_handles_mut(&mut self) -> &mut [*const ffi::c_void] {
		if matches!(self.num_entries, 0) {
			&mut []
		} else {
			unsafe { slice::from_raw_parts_mut(self.type_handles, self.num_entries as usize) }
		}
	}

	#[must_use]
	pub const fn type_ids(&self) -> &[TypeId<'a>] {
		if matches!(self.num_entries, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.type_ids, self.num_entries as usize) }
		}
	}

	#[must_use]
	pub unsafe fn get_type_handle_unchecked(&self, idx: u32) -> *const ffi::c_void {
		unsafe { *self.type_handles.offset(idx as isize) }
	}

	#[must_use]
	pub fn get_type_handle(&self, idx: u32) -> Option<*const ffi::c_void> {
		if idx < self.num_entries {
			Some(unsafe { self.get_type_handle_unchecked(idx) })
		} else {
			None
		}
	}

	pub unsafe fn get_type_handle_unchecked_mut(&mut self, idx: u32) -> &mut *const ffi::c_void {
		unsafe { &mut *self.type_handles.offset(idx as isize) }
	}

	pub fn get_type_handle_mut(&mut self, idx: u32) -> Option<&mut *const ffi::c_void> {
		if idx < self.num_entries {
			Some(unsafe { self.get_type_handle_unchecked_mut(idx) })
		} else {
			None
		}
	}

	pub fn type_names(&self) -> impl Iterator<Item = &str> {
		let type_names = if matches!(self.num_entries, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.type_names, self.num_entries as usize) }
		};

		type_names
			.iter()
			.map(|n| unsafe { str::from_utf8_unchecked(CStr::from_ptr(*n).to_bytes()) })
	}
}

impl Serialize for TypeLut<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		struct Elem<'a> {
			name: &'a str,
			r#type: &'a TypeId<'a>,
		}

		let mut s = serializer.serialize_seq(Some(self.num_entries as usize))?;

		for (ty, .., name) in self.iter() {
			s.serialize_element(&Elem { name, r#type: ty })?;
		}

		s.end()
	}
}
