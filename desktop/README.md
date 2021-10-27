# Desktop for Wikit

# Introduction

The desktop of wikit will be platform agnostic, as a result, linux, macos and windows will be
supported.

The ui part will be written using modern web technology, to be concise, it will use
[svelte](https://svelte.dev/) framewrok.

The backend will be written using Rust programming language with [tauri](https://github.com/tauri-apps/tauri) framewrok.

# Development

Before doing development, ensure you have `Rust` and `Node` installed. If you do not known how to install,
google it. If these tools are ready, following below command to start development

- install dependencies

        npm i

- start web dev

    In one terminal, run

        npm run dev

- start tauri dev

    In another terminal, run

        npm run tauri dev

    or run the following command if you have installed tauri-cli using `cargo install tauri-cli`

        cargo tauri dev

Start coding!
