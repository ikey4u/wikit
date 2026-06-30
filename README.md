<p align="center">
  <img src="https://github.com/ikey4u/wikit/blob/master/desktop/electron/icons/128x128@2x.png?raw=true" alt="Bootsmind Wikit Application Icon"/>
</p>
<p align="center">
  Bootsmind Wikit is a free and open-source dictionary program that enables you to translate words across different languages
</p>

<hr/>

Bootsmind Wikit contains the following components

- Bootsmind Wikit CLI

    A CLI tool which can be used to create `.wikit` dictionaries from plain text or other dictionary formats.

- Bootsmind Wikit Desktop

    Desktop application for Windows, Linux and macOS built with Electron and a Rust NAPI native module.

    ![Bootsmind Wikit main screen](./docs/imgs/wikit-main-screen.jpg "Bootsmind Wikit main screen")

- Bootsmind Wikit Mobile

    Mobile application for Android; it's under planning.

# Installation and Usage

You can download Bootsmind Wikit CLI and Bootsmind Wikit Desktop from the [Release](https://github.com/ikey4u/wikit/releases) page.

To install dictionaries, see the [Wikit Introduction](https://github.com/ikey4u/wikit/wiki) for details.

For Linux users, you can create a file at `~/.local/share/applications/com.bootsmind.wikit.desktop`
(create the path if it does not exist) with the following content:

    #!/usr/bin/env xdg-open

    [Desktop Entry]
    Name=Bootsmind Wikit
    Comment=A universal dictionary
    Path=/path/to/wikit
    Exec=/path/to/wikit/bootsmind-wikit.AppImage
    Terminal=false
    Type=Application
    Categories=Utility;
    Keywords=dictionary;dict;

Change `Path` and `Exec` to your own install location, then run:

    update-desktop-database ~/.local/share/applications

After that you can open Bootsmind Wikit from your application launcher.

# Development

To develop Bootsmind Wikit CLI:

    cd cli
    cargo build

To develop Bootsmind Wikit Desktop:

    cd desktop/electron
    npm install
    npm start

To develop Bootsmind Wikit Mobile:

    cd android
    make start

# Building

To build Bootsmind Wikit CLI:

    cd cli
    cargo build --release

To build Bootsmind Wikit Desktop:

    mise pkg

To build Bootsmind Wikit Mobile:

    cd android
    make release

# License

[MIT](./LICENSE)
