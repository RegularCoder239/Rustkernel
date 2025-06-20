mod vecchunk;
mod vec;
mod veciter;
mod stackvec;

pub use vec::{
	Vec,
	VecBase
};
pub use vecchunk::{
	VecChunk
};
pub use veciter::{
	VecIter,
	VecIterMut,
	VecIterNonRef
};
pub use stackvec::{
	StackVec
};

#[macro_export]
macro_rules! vec {
	() => {
		crate::std::Vec::new()
	};
	($($elements: expr),+ $(,)?) => {
		crate::std::Vec::from_slice([$($elements),+].as_slice())
	};
}

#[macro_export]
macro_rules! stack_vec {
	($($elements: expr),+ $(,)?) => {
		crate::std::StackVec::from_slice([$($elements),+].as_slice())
	};
}
