use rs_tml::element::{Node, element};
use rs_tml_macro::rstml;
#[test]
fn test_empty() {
    let document = rstml! {};
    assert!(document.children.is_empty())
}

#[test]
fn test_if_block() {
    let text = "Hello, World!";
    let document = rstml! {
        if true {
            "{text}"
        }
    };
    assert_eq!(document.children.len(), 1);
    assert_eq!(document.children[0], Node::text(text));
}

#[test]
fn test_for_block() {
    let items = vec!["Item 1", "Item 2", "Item 3"];
    let document = rstml! {
        for item in items {
            li { "{item}" }
        }
    };
    assert_eq!(document.children.len(), 3);
    for (i, child) in document.children.iter().enumerate() {
        assert_eq!(
            child,
            &element("li")
                .with_child(format!("Item {}", i + 1))
                .into_node()
        );
    }
}

#[test]
fn test_match_block() {
    let value = 2;
    let document = rstml! {
        match value {
            1 => p { "One" },
            2 => p { "Two" },
            _ => p { "Other" },
        }
    };
    assert_eq!(document.children.len(), 1);
    assert_eq!(
        document.children[0],
        element("p").with_child("Two").into_node()
    );
}

#[test]
fn test_nested_elements() {
    let document = rstml! {
        div {
            h1 { "Title" }
            p { "This is a paragraph." }
        }
    };
    assert_eq!(document.children.len(), 1);
    let expected = element("div")
        .with_child(element("h1").with_child("Title"))
        .with_child(element("p").with_child("This is a paragraph."))
        .into_node();
    assert_eq!(document.children[0], expected);
}

#[test]
fn test_class_attribute_shorthand() {
    let document = rstml! {
        div {
            .class-name
            "Content"
        }
    };
    assert_eq!(document.children.len(), 1);
    let expected = element("div")
        .with_key_value("class", "class-name")
        .with_child("Content")
        .into_node();
    assert_eq!(document.children[0], expected);
}

#[test]
fn test_id_attribute_shorthand() {
    let document = rstml! {
        div {
            #unique-id
            "Content"
        }
    };
    assert_eq!(document.children.len(), 1);
    let expected = element("div")
        .with_key_value("id", "unique-id")
        .with_child("Content")
        .into_node();
    assert_eq!(document.children[0], expected);
}

#[test]
fn test_attribute_spread() {
    let attrs = vec![("class", "btn"), ("id", "submit-button")];
    let document = rstml! {
        button {
            ..attrs
            "Submit"
        }
    };
    let button = element("button")
        .with_key_value("class", "btn")
        .with_key_value("id", "submit-button")
        .with_child("Submit")
        .into_node();
    assert_eq!(document.children.len(), 1);
    assert_eq!(document.children[0], button);
}

#[test]
fn test_dynamic_attribute_key() {
    let attr_name = "data-info";
    let document = rstml! {
        div {
            .*attr_name = "some data"
            "Content"
        }
    };
    let expected = element("div")
        .with_key_value(attr_name, "some data")
        .with_child("Content")
        .into_node();
    assert_eq!(document.children.len(), 1);
    assert_eq!(document.children[0], expected);
}

#[test]
fn test_dynamic_attribute_value() {
    let attr_value = "dynamic-value";
    let document = rstml! {
        div {
            .data-attr = attr_value
            "Content"
        }
    };
    let expected = element("div")
        .with_key_value("data-attr", attr_value)
        .with_child("Content")
        .into_node();
    assert_eq!(document.children.len(), 1);
    assert_eq!(document.children[0], expected);
}
