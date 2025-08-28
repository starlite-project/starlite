use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use once_cell::sync::OnceCell;
use serde::Serialize;

use crate::{Guid, static_type_map::StaticTypeMap};

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct PointerTypeId<'a> {
	pub pointee: &'a TypeId<'a>,
	pub mutable: bool,
}

impl Display for PointerTypeId<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(if self.mutable { "*mut " } else { "*const " })?;

		Display::fmt(&self.pointee, f)
	}
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ArrayTypeId<'a> {
	pub element: &'a TypeId<'a>,
}

impl Display for ArrayTypeId<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_char('[')?;
		Display::fmt(&self.element, f)?;
		f.write_char(']')
	}
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum TypeId<'a> {
	Concrete(Guid),
	Pointer(PointerTypeId<'a>),
	Array(ArrayTypeId<'a>),
}

impl Display for TypeId<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Concrete(guid) => Display::fmt(&guid, f),
			Self::Pointer(ptr) => Display::fmt(&ptr, f),
			Self::Array(array) => Display::fmt(&array, f),
		}
	}
}

impl From<Guid> for TypeId<'_> {
	fn from(value: Guid) -> Self {
		Self::Concrete(value)
	}
}

impl<'a> From<PointerTypeId<'a>> for TypeId<'a> {
	fn from(value: PointerTypeId<'a>) -> Self {
		Self::Pointer(value)
	}
}

impl<'a> From<ArrayTypeId<'a>> for TypeId<'a> {
	fn from(value: ArrayTypeId<'a>) -> Self {
		Self::Array(value)
	}
}

unsafe impl Send for TypeId<'_> {}
unsafe impl Sync for TypeId<'_> {}

pub trait HasStaticTypeId {
	fn type_id() -> &'static TypeId<'static>;
}

impl<T> HasStaticTypeId for *const T
where
	T: ?Sized + HasStaticTypeId + 'static,
{
	fn type_id() -> &'static TypeId<'static> {
		static VALUE: OnceCell<StaticTypeMap<TypeId<'static>>> = OnceCell::new();
		let map = VALUE.get_or_init(Default::default);
		map.call_once::<T>(|| {
			PointerTypeId {
				pointee: T::type_id(),
				mutable: false,
			}
			.into()
		})
	}
}

impl<T> HasStaticTypeId for *mut T
where
	T: ?Sized + HasStaticTypeId + 'static,
{
	fn type_id() -> &'static TypeId<'static> {
		static VALUE: OnceCell<StaticTypeMap<TypeId<'static>>> = OnceCell::new();
		let map = VALUE.get_or_init(Default::default);
		map.call_once::<T>(|| {
			PointerTypeId {
				pointee: T::type_id(),
				mutable: true,
			}
			.into()
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::{ArrayTypeId, HasStaticTypeId, PointerTypeId, PrimitiveType, TypeId};

	#[test]
	fn display() {
		assert_eq!(i32::type_id().to_string(), i32::guid().to_string());
		assert_eq!(f64::type_id().to_string(), f64::guid().to_string());
		assert_eq!(
			std::ffi::c_void::type_id().to_string(),
			std::ffi::c_void::guid().to_string()
		);

		let i32_type_id = i32::type_id();
		assert_eq!(
			TypeId::Pointer(PointerTypeId {
				pointee: i32_type_id,
				mutable: false
			})
			.to_string(),
			format!("*const {}", i32::guid())
		);
		assert_eq!(
			TypeId::Pointer(PointerTypeId {
				pointee: i32_type_id,
				mutable: true
			})
			.to_string(),
			format!("*mut {}", i32::guid())
		);

		assert_eq!(
			TypeId::Array(ArrayTypeId {
				element: i32_type_id
			})
			.to_string(),
			format!("[{}]", i32::guid())
		);
	}
}
