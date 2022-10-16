# Wikit - A universal dictionary

*THESE CODES ARE UNDER HEAVY REFACTOR, THEY MAY NOT WORK AS EXPECTED*

# What is it?

To be short, Wikit is a dictionary suite for human in [FOSS](https://en.wikipedia.org/wiki/Free_and_open-source_software) style.

So what are planned to be included? The goals of this project are to make

- **A CLI tool to deal with a variety of dictionary formats**

- **Desktop application for Windows, Linux and MacOS**

    The desktop client is developed using [tauri](https://tauri.studio/en/) and [react](https://reactjs.org/).

    ![lookup ui](./docs/imgs/lookup.jpg "lookup ui")

- **A dictionary server**

    It is usable for now, but there are many things to be improved.

# Installation and Usage

There are two tools provided by wikit, one is `Wikit Command Line` (abbreviated as wikit), the other is `Wikit Desktop`.
The former is used to create, unpack, parse dictionary, or even used as a dictionary server, the
latter is used as a dictionary client which you can lookup words from.

You can download them from [Release](https://github.com/ikey4u/wikit/releases) page.

For Linux user, you can create a file in path `~/.local/share/applications/com.zhqli.wikit.desktop`
(if the path does not exist, create it) with the following content (you must change `Path` and  `Exec` to your own):

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

And run `update-desktop-database`, after that you can open wikit desktop from your dash.

    update-desktop-database ~/.local/share/applications

**For Windows user, you must additionally install [webview2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section) and [vc_redist.x86](https://aka.ms/vs/17/release/vc_redist.x86.exe) or [vc_redist.x64](https://aka.ms/vs/17/release/vc_redist.x64.exe).**

## Creating dictionary

- Prepare your dictionary

    Assume that your dictionary source is put under directory `my_awesome_dict`, it should meet the
    following organization

        my_awesome_dict/
        +-- audios/

            optional, any audios should be put here

        +-- images/

            optional, any images should be put here

        +-- header.wikit.txt

            required, this is the header of your dictionary. It contains basic information,
            style, script for your dictionary.

        +-- body.wikit.txt

            required, this is the body (words and meanings) of your dictionary

        +-- preview.wikit.txt

            optional, preview item in your dictionary

    Text file with `.wikit.txt` suffix has specific format:

        (
            <PARAMS>
        ) {
            <CONTENT>
        }

    If you have experiences with mainstream programming language such as C/C++, Python,
    JavaScript, you may feel familiar with the format. It likes a function definition but without
    function name. wikit has extended the style, the `<PARAMS>` is a `json5` body with
    following keys required:

    - `name`: This entry name
    - `type`: This entry type
    - `mime`: The content type of this entry, a subset from [HTTP MIME](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)

    Note that the format is strict, each of the following line must hold one single line and have no prefix
    spaces:

        (
        ) {
        }

    And more, `<PARAMS>` and `<CONTENT>` must have a indent of zero or 4 more spaces.

    <details>

    <summary>Here is a brief example, click to expand.</summary>

    - `header.wikit.txt`

        ```
        (
            "name": "wikit example dictionary",
            "type": "info",
            "mime": "application/toml",
        ) {
            desc = '''
            This is just a wikit example dictionary, nothing more.
            '''

            author = "wikit author"
        }

        (
            "type": "js",
            "name": "script.js",
            "mime": "text/javascript",
        ) {
            // put your js script here
        }


        (
            "type": "css",
            "name": "style.css",
            "mime": "text/css",
        ) {
            /* put you css style here */
        }
        ```

    - `body.wikit.txt`

        ```
        (
            "type": "word",
            "name": "cat",
            "mime": "text/html",
        ) {
            <div class="meaning">
              <h2>cat</h2>
            </div>
        }
        ```

    - `preview.wikit.txt`

        ```
        (
            "type": "word",
            "name": "cat",
            "mime": "text/html",
        ) {
            <div class="meaning">
              <h2>cat</h2>
            </div>
        }
        ```

    A full example can be found at [wikit/examples/dict](https://github.com/ikey4u/wikit/tree/master/examples/dict).

    </details>

- Preview your dictionary

    Add words you want to preview into `preview.wikit.txt`, and run the following command

        wikit preview /path/to/my_awesome_dict

- Build your dictionary

    Congratulations! After some hard work, you have prepared your dictionary. Its time to build,
    just run the following command and your are done

        wikit dict --create -o dict.wikit /path/to/my_awesome_dict

- Bonus

    What's more, wikit also support you create wikit dictionary directly from MDX file as below

        wikit dict --create -o /path/to/dict.wikit /path/to/dict.mdx

## Configuring dictionary

Let's assume you have a wikit dictionary located at `/path/to/dict.wikit`, you should create a
configuration file in the following location

```
macos: ${HOME}/Library/Application Support/wikit/wikit.toml
linux: ${HOME}/.config/wikit/wikit.toml
windows: C:\Users\YOUNAME\AppData\Roaming\wikit\wikit.toml
```

The content of `wikit.toml` looks like

```
[cltcfg]
uris = [
    "file:///path/to/dict.wikit",
]

[srvcfg]
uris = [
]
host = "0.0.0.0"
port = 8888
```

If you use wikit desktop, you should focus on the section `[cltcfg]` and do not touch `[srvcfg]` section.

`uris` can be path to your wikit dictionary (the path must begin with `file://`) or API address (must
starts with http or https) that your wikit server provides, such as `http://192.168.1.8:8888`.
The API address should be IP address for now, domain support will be added in future.

If run wikit as as a dictionary server, you should focus on `[srvcfg]` section.
`uris` are the same as `[cltcfg]`, `host` and `port` will be the address your dictionary server
listens to.

## Using dictionary

Everything is done, open `Wikit Desktop` and start to lookup.

If you add, delete or change the wikit dictionary, remember to restart `Wikit Desktop`.

# Developement

Install dependencies following the [tauri guide](https://tauri.app/v1/guides/getting-started/prerequisites/), then install tauri-cli

    cargo install --version 1.1.1 tauri-cli

and check the version

    cargo tauri --version

Create a file named `.env` under directory `desktop/ui` with content

    BROWSER=none
    PORT=8080

Start developent using the following command in `desktop/tauri` directory

    cargo tauri dev

# Building

To build wikit command line

    cd cli
    cargo build --release

To build wikit desktop

    cd desktop
    cargo tauri build

You can find the generated files in `target/release`.

# License

[MIT](./LICENSE)
