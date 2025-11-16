# rs-tml

A Rust-like markup language and macro system for generating markup code.

## Features

- **Intuitive HTML-like syntax** - Write markup using familiar HTML element names with Rust-like syntax
- **Styling support** - Define inline styles with a clean block syntax
- **CSS class shortcuts** - Use `.classname` shorthand for classes and `#id` for IDs
- **Attribute binding** - Set attributes with `.attr = "value"` syntax
- **Dynamic attributes** - Use variables and expressions for attribute names with `.*name` syntax
- **Conditional rendering** - Use `if/else if/else` statements to conditionally render elements
- **Pattern matching** - Use `match` expressions to render different content based on patterns
- **Iterators** - Loop over collections with `for` loops to generate repeated elements
- **String interpolation** - Embed expressions directly in text with `"{expr}"` syntax
- **Component expansion** - Include child components with `*child` syntax
- **Spread operators** - Expand iterators into multiple attributes with `..*attrs`
- **Comments** - Single-line `//` and multi-line `/* */` comments supported

An example document is available in the [intro](./intro.rstml) file.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE.md) file for details.
