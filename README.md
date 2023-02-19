<p align="center">
  <img src="https://github.com/ikey4u/wikit/blob/master/desktop/tauri/icons/128x128@2x.png?raw=true" alt="Wikit Application Icon"/>
</p>
<p align="center">
  Wikit is a free and open-source dictionary program that enable you translate word for different languages
</p>

<hr/>

wikit contains the following components

- Wikit CLI

    A CLI tool which can be used to create wikit dictionary from plain text or other dictionary format.

- Wikit Desktop

    Desktop application for Windows, Linux and MacOS which is developed using [tauri](https://tauri.studio/en/) and [yew](https://yew.rs/).

    ![wikit main screen](./docs/imgs/wikit-main-screen.jpg "wikit main screen")

- Wikit Mobile

    Mobile application for Android, it's under planning.

# Installation and Usage

You can download Wikit CLI and Wikit Desktop from [Release](https://github.com/ikey4u/wikit/releases) page.

To install dictionary, see [Wikit Introduction](https://github.com/ikey4u/wikit/wiki) for detail.

For Linux user, you can create a file in path `~/.local/share/applications/com.zhqli.wikit.desktop`
(create if the path does not exist) with the following content:

    #!/usr/bin/env xdg-open

    [Desktop Entry]
    Name=Wikit Desktop
    Comment=A universal dictionary
    Path=/path/to/wikit
    Exec=/path/to/wikit/wikit-desktop.AppImage 
    Terminal=false
    Type=Application
    Categories=Utility;
    Keywords=dictionary;dict;

You should change `Path` and  `Exec` to your own, and run `update-desktop-database`

    update-desktop-database ~/.local/share/applications

after that you can open wikit desktop from your dash.                               

If you are bother with the manual installation on linux, you can install it from
[flathub](https://flathub.org/apps/details/com.zhqli.wikit).

**For Windows user, you must additionally install [webview2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section) and [vc_redist.x86](https://aka.ms/vs/17/release/vc_redist.x86.exe) or [vc_redist.x64](https://aka.ms/vs/17/release/vc_redist.x64.exe).**

# Developement

Install following tools

    cargo install tauri-cli trunk

    # rust for android target
    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

    # sdkmanager is from cmdline-tools https://developer.android.com/studio/command-line
    sdkmanager "platforms;android-31" "platform-tools" "build-tools;31.0.0" "cmake;3.22.1" "ndk;25.1.8937393"

Create a file named `.env` under directory `desktop/ui` with content

    BROWSER=none
    PORT=8080

To develop wikit CLI

    cd cli
    cargo build

To develop wikit desktop

    cd desktop
    cargo tauri dev

To develop wikit mobile

    cd android
    make start

# Building

To build wikit CLI

    cd cli
    cargo build --release

To build wikit desktop

    cd desktop
    cargo tauri build

To build wikit mobile

    cd android
    make release

# License

[MIT](./LICENSE)
