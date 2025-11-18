pub mod attribute;
pub mod block;
pub mod element;
pub mod node;
pub mod tag;
pub mod text;

pub mod prelude {
    use super::{attribute, block, element, node, tag, text};
    pub use attribute::Attribute;
    pub use block::Block;
    pub use element::{Element, element};
    pub use node::Node;
    pub use tag::Tag;
    pub use text::Text;
}
