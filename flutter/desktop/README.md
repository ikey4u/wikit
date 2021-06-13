# Desktop UI based on [NativeShell](https://github.com/nativeshell/nativeshell)

Flutter desktop does not support embedded webview for now, but wikit requries that. UI development
will continue when the requirements is meet.

## Prerequisites

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. [Install Flutter](https://flutter.dev/docs/get-started/install)
3. [Enable Flutter desktop support](https://flutter.dev/desktop#set-up)
4. Switch to Fluttter Master (`flutter channel master; flutter upgrade`)

If you have some troubles when building, see discussion [here](https://github.com/nativeshell/examples/issues/4).

## Getting Started

Launch it with `make run`.

To debug or hot reload dart code, run `make reload` on linux.
