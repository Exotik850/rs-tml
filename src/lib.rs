use pastey::paste;

pub struct Tag<'a> {
    name: &'a str,
}

macro_rules! tag {
    ($($name:ident)+) => {
        $(
          paste! {
            pub const [<$name:upper>]: Tag<'a> = Tag::new(stringify!([<$name:lower>]));
          }
        )+
    };
}

impl<'a> Tag<'a> {
    const fn new(name: &'a str) -> Self {
        Tag { name }
    }

    const fn as_str(&self) -> &'a str {
        self.name
    }

    tag!(div span p a img ul li table tr td th header footer nav section article main aside form input button label select option textarea);
}

impl std::fmt::Display for Tag<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct Attribute<'a> {
    key: &'a str,
    value: &'a str,
}

impl<'a> Attribute<'a> {
    const fn new(key: &'a str, value: &'a str) -> Self {
        Attribute { key, value }
    }
}

pub struct Text<'a> {
    content: &'a str,
}

pub struct Comment<'a> {
    content: &'a str,
}

// Generic Element struct that can hold different types of children
//
// Typically holds either Text, Comment, or a Vec<Element>, or () for no children
pub struct Element<'a, T = ()> {
    name: Tag<'a>,
    attributes: Vec<Attribute<'a>>,
    children: T,
}
