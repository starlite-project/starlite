use std::{any::TypeId, cell::RefCell, collections::HashMap};

use parking_lot::ReentrantMutex;

#[repr(transparent)]
pub struct StaticTypeMap<T>
where
	T: ?Sized + 'static,
{
	map: ReentrantMutex<RefCell<HashMap<TypeId, &'static T>>>,
}

impl<T: 'static> StaticTypeMap<T> {
	pub fn call_once<Type>(&'static self, f: impl FnOnce() -> T) -> &'static T
	where
		Type: ?Sized + 'static,
	{
		let map = self.map.lock();
		if let Some(r) = map.borrow().get(&TypeId::of::<Type>()) {
			return r;
		}

		let reference = Box::leak(Box::new(f()));

		let old = map.borrow_mut().insert(TypeId::of::<Type>(), reference);
		assert!(old.is_none(), "static type map value was reinitialized");

		reference
	}
}

impl<T> Default for StaticTypeMap<T>
where
	T: ?Sized + 'static,
{
	fn default() -> Self {
		Self {
			map: ReentrantMutex::new(RefCell::new(HashMap::new())),
		}
	}
}
