mod container;
mod product;
mod raw_parser;
mod tags;
mod vendor;

pub use container::{ProductContainer, SuperAlloc};
pub use product::Product;
pub use raw_parser::{optimize, RawProduct};
