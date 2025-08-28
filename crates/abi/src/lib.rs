#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod assembly_info;
mod dispatch_table;
mod function_info;
mod module_info;
mod primitive;
mod static_type_map;
mod struct_info;
#[cfg(test)]
mod test_utils;
mod r#type;

use std::{
	ffi::CStr,
	fmt::{Display, Formatter, Result as FmtResult},
};

use serde::{Serialize, Serializer};
use serde_repr::Serialize_repr;

pub use self::{
	assembly_info::AssemblyInfo,
	dispatch_table::DispatchTable,
	function_info::{FunctionDefinition, FunctionPrototype, FunctionSignature},
	module_info::ModuleInfo,
	primitive::PrimitiveType,
	static_type_map::StaticTypeMap,
	struct_info::{StructDefinition, StructMemoryType},
	r#type::{
		ArrayTypeId, HasStaticTypeId, HasStaticTypeName, PointerTypeId, TypeDefinition,
		TypeDefinitionData, TypeId, TypeLut,
	},
};

pub const ABI_VERSION: u32 = 3_00;
pub const GET_INFO_FN_NAME: &str = "get_info";
pub const GET_VERSION_FN_NAME: &str = "get_version";
pub const SET_ALLOCATOR_HANDLE_FN_NAME: &str = "set_allocator_handle";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Guid(pub [u8; 16]);

impl Guid {
	#[must_use]
	pub const fn from_str(s: &str) -> Self {
		Self(extendhash::md5::compute_hash(s.as_bytes()))
	}

	#[must_use]
	pub const fn from_cstr(s: &CStr) -> Self {
		Self(extendhash::md5::compute_hash(s.to_bytes()))
	}
}

impl Display for Guid {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		const fn format_hyphenated(src: [u8; 16]) -> [u8; 36] {
			const LUT: [u8; 16] = [
				b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
				b'e', b'f',
			];

			let groups = [(0, 8), (9, 13), (14, 18), (19, 23), (24, 36)];
			let mut dst = [0; 36];

			let mut group_idx = 0;
			let mut i = 0;
			while group_idx < 5 {
				let (start, end) = groups[group_idx];
				let mut j = start;
				while j < end {
					let x = src[i];
					i += 1;

					dst[j] = LUT[(x >> 4) as usize];
					dst[j + 1] = LUT[(x & 0x0f) as usize];
					j += 2;
				}

				if group_idx < 4 {
					dst[end] = b'-';
				}

				group_idx += 1;
			}

			dst
		}

		let hyphenated = format_hyphenated(self.0);

		let hyphenated = unsafe { std::str::from_utf8_unchecked(&hyphenated) };

		f.write_str(hyphenated)
	}
}

impl Serialize for Guid {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr)]
#[repr(u8)]
pub enum Privacy {
	Public = 0,
	Private,
}
