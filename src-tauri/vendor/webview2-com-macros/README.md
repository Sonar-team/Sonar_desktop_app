# webview2-com-macros
This crate implements macros which generate callback implementations for WebView2 COM APIs. In turn, it injects references to the `#[implement]` macro from the [Windows](https://github.com/microsoft/windows-rs) crate, along with a lot of boilerplate code to handle varying parameter types.

## Getting Started
This crate is only intended for use in [webview2-com](https://crates.io/crates/webview2-com).