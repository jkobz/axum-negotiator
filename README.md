<a href="https://crates.io/crates/axum-content-negotiation">
    <img src="https://img.shields.io/crates/v/axum-content-negotiation.svg" />
</a>

Content-negotiation utilities. Provides [Negotiate] trait and *Tower*-compatible
[Layer] and [Service] that guarantee an **HTTP** response to be serialized with
respect to the *valid* *accepted* **media-type**. Exposes content-negotiation
headers via [header], **media-type** via [media].

---

### Usage

```toml
[dependencies]
axum-content-negotiation = "1.0.0"
```
