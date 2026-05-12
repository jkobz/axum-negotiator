<a href="https://crates.io/crates/axum-negotiator">
    <img src="https://img.shields.io/crates/v/axum-negotiator.svg" />
</a>

# **Content-negotiation** utilities.

## Motivation

Backend services often are required to support multiple representations of the same data
i.e. `text/html` and `application/json`. This implies deciding how incoming request payloads
are parsed via [`Content-Type`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Type),
and how the response should be serialized  via [`Accept`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Accept).
Most frameworks handle this manually, leading to:
**HTTP** request headers pattern-matching.

Despite sounding quite simple, it gets *tedious* and *error-prone* quite soon, if implemented manually.
One would achieve that with *repeated* pattern-matching and extraction logic, *inconsistent* error handling,
and *tight coupling* between business-logic and **content-negotiation** strategy.

## Abstraction model

- [Negotiate] serializes response, given the **media-type** context
- [media::Stateful] converts *content-negotiated* value into output
- [media::Either] resolves branching negotiation
- [Payload] simplifies request-side negotiation
- [Layer] enforcement boundary

## Request content-negotiation

Code below becomes *repetitive* across handlers:

```rust
match content_type {
    Json => extract_json(req),
    Form => extract_form(req),
}
```

### Solution

[`Payload`] deserializes data based on `Content-Type` **HTTP** request header.

```rust
use axum::RequestExt;

struct MyPayload;
struct NotSupportedError;
struct BadRequestError;

req.extract::<
    Payload<
        MyPayload,
        media::Either<media::Form, media::Json>,
        NotSupportedError,
        BadRequestError,
    >,
    _
>().await?;
```

## Response content-negotiation

Code below becomes *repetitive* across handlers and middlewares:

```rust
match accept {
    Html => render_html(),
    Json => render_json(),
}
```

### Solution

[`Negotiate`] trait provides an arbitrary type with `into_response` method that
serializes the **HTTP** response based on the **media-type**.

Use [`Either`] to represent multiple supported response **media-types**:

```rust
media::Either<media::Html, media::Json>
```

**Content-negotiation** is enforced at the middleware layer:

```rust
ServiceBuilder::new()
    .layer(
        Layer::<
            media::Either<media::Html, media::Json>,
            NotAcceptableError,
            BadRequestError,
            (),
        >::new(),
    )
```

This approach guarantees:

1. Request is checked against supported **media-types**;
2. Response is always serialized in a *valid* **format**;
3. *Unsupported* clients receive structured errors
