use std::{ffi::c_void, slice};

use serde::{Serialize, Serializer, ser::SerializeStruct as _};

use super::FunctionPrototype;

#[repr(C)]
pub struct DispatchTable<'a> {
	pub(crate) prototypes: *const FunctionPrototype<'a>,
	pub(crate) fn_ptrs: *mut *const c_void,
	pub num_entries: u32,
}

#[allow(clippy::iter_on_empty_collections)]
impl<'a> DispatchTable<'a> {
	pub fn iter_mut(
		&mut self,
	) -> impl Iterator<Item = (&mut *const c_void, &FunctionPrototype<'a>)> {
		if matches!(self.num_entries, 0) {
			([]).iter_mut().zip(([]).iter())
		} else {
			let ptrs =
				unsafe { slice::from_raw_parts_mut(self.fn_ptrs, self.num_entries as usize) };
			let signatures =
				unsafe { slice::from_raw_parts(self.prototypes, self.num_entries as usize) };

			ptrs.iter_mut().zip(signatures.iter())
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = (&*const c_void, &FunctionPrototype<'a>)> {
		if matches!(self.num_entries, 0) {
			([]).iter().zip(([]).iter())
		} else {
			let ptrs =
				unsafe { slice::from_raw_parts_mut(self.fn_ptrs, self.num_entries as usize) };
			let signatures =
				unsafe { slice::from_raw_parts(self.prototypes, self.num_entries as usize) };

			ptrs.iter().zip(signatures.iter())
		}
	}

	pub const fn ptrs_mut(&mut self) -> &mut [*const c_void] {
		if matches!(self.num_entries, 0) {
			&mut []
		} else {
			unsafe { slice::from_raw_parts_mut(self.fn_ptrs, self.num_entries as usize) }
		}
	}

	#[must_use]
	pub const fn prototypes(&self) -> &[FunctionPrototype<'a>] {
		if matches!(self.num_entries, 0) {
			&[]
		} else {
			unsafe { slice::from_raw_parts(self.prototypes, self.num_entries as usize) }
		}
	}

	#[must_use]
	pub unsafe fn get_ptr_unchecked(&self, idx: u32) -> *const c_void {
		unsafe { *self.fn_ptrs.offset(idx as isize) }
	}

	#[must_use]
	pub fn get_ptr(&self, idx: u32) -> Option<*const c_void> {
		if idx < self.num_entries {
			Some(unsafe { self.get_ptr_unchecked(idx) })
		} else {
			None
		}
	}

	pub unsafe fn get_ptr_unchecked_mut(&mut self, idx: u32) -> &mut *const c_void {
		unsafe { &mut *self.fn_ptrs.offset(idx as isize) }
	}

	pub fn get_ptr_mut(&mut self, idx: u32) -> Option<&mut *const c_void> {
		if idx < self.num_entries {
			Some(unsafe { self.get_ptr_unchecked_mut(idx) })
		} else {
			None
		}
	}
}

impl Serialize for DispatchTable<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = serializer.serialize_struct("DispatchTable", 1)?;
		s.serialize_field("prototypes", self.prototypes())?;
		s.end()
	}
}
