# streamdown-plugin

Plugin system for the [streamdown](https://crates.io/crates/streamdown) streaming markdown renderer.

## Overview

Extensibility framework for streamdown:

- **Text transformers** - Process text before/after rendering
- **LaTeX support** - Convert LaTeX math to Unicode symbols
- **Custom handlers** - Add new block types and behaviors
- **Pipeline architecture** - Composable plugin chains

## Built-in Plugins

### LaTeX to Unicode

Converts LaTeX math notation to Unicode symbols:

```
\alpha  -> α
\beta   -> β
\sum    -> Σ
\int    -> ∫
\infty  -> ∞
```

## Usage

```toml
[dependencies]
streamdown-plugin = "0.1"
```

```rust
use streamdown_plugin::{PluginManager, LatexPlugin};

let mut plugins = PluginManager::new();
plugins.register(LatexPlugin::new());

let text = r"The formula is \alpha + \beta = \gamma";
let processed = plugins.process(text);
// Output: "The formula is α + β = γ"
```

## Part of Streamdown

This is a component of [streamdown-rs](https://github.com/fed-stew/streamdown-rs), a streaming markdown renderer for modern terminals.

## License

MIT
