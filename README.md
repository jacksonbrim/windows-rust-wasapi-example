# Windows Audio Device Example in Rust
This is a basic example demonstrating the usage of the Windows Audio Session Api (WASAPI) in Rust.

This example shows how to use the Windows Api for rust to enumerate over Audio Devices,
get audio device details, formats, etc. and ultimately to render an audio output
stream that plays a single tone for 5 seconds.

The original C++ example, can be found here:
https://learn.microsoft.com/en-us/windows/win32/coreaudio/rendering-a-stream

[Windows Crate Docs hosted by Microsoft](https://microsoft.github.io/windows-docs-rs/doc/windows/index.html)

## Challenges working with the Rust APIs for Windows
Working with the Windows Rust Apis can be a bit challenging at the moment as you must 
constantly refer back and forth between the Cpp documentation and the Rust documentation to infer usage, flags, etc.

The feature flags for modules are not annotated in the official documentation,
due to 
searching the particular Struct, Trait, etc. at the following location:

https://microsoft.github.io/windows-rs/features/#/0.56.0

This leads to some unfortunate scenarios, where you're certain that you've
compiled the crate with the appropriate feature flags, leaving you scratching
your head when something doesn't work.

### [Common Issues with Feature Flag Searching](./resources/feature_flag_searching.md)


