# Wikit - A universal dictionary

# What is it?

To be short, Wikit is a dictionary suite for human in [FOSS](https://en.wikipedia.org/wiki/Free_and_open-source_software) style.

So what are planned to be included? The goal of this project is to make

- A CLI tool to deal with a variety of dictionary formats

- Desktop application for Windows, Linux and MacOS

- A dictionary server

- Mobile application for Android and iOS

# What is the project status?

- CLI TOOL

    The CLI tool can parse/create MDX dictionary and create macos dictionary, see its usage in
    `Installation and Usage` section.

- DESKTOP APPLICATION

    It is developed based on [tauri](https://tauri.studio/en/) and [svelte](https://svelte.dev/), working in progress.

- DICTIONARY SERVER

    A quick and dirty demo is developed in `core/src/router.rs`, and here is the
    [demo](http://106.53.152.194/wikit/).

- MOBILE APPLICATION

    No action for now.

# Installation and Usage

Download the tool from [release](https://github.com/ikey4u/wikit/releases) page, decompress the
release packege and just fireup the tool `wikit`, you will see detail help information, for example

    wikit 0.2.0-beta.1
    ikey4u <pwnkeeper@gmail.com>
    A universal dictionary - Wikit

    USAGE:
        wikit [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        dict      Process dictionary file
        help      Prints this message or the help of the given subcommand(s)
        server    Run wikit as an API server

There are serveral subcommands: `dict` and `server`. However `server` is not stable for now,
please use it carefully. For `dict` subcommand, you can print its help information using following command

    wikit dict

An example output is showed below

    Process dictionary file

    USAGE:
        wikit dict [FLAGS] [OPTIONS] <input>

    FLAGS:
        -c, --create     Create dictionary file
        -h, --help       Prints help information
            --info       Dump basic information of dictionary file
        -V, --version    Prints version information

    OPTIONS:
            --css <css>          Path of the CSS file
            --meta <metafile>    You could specify a meta file when create dictionary file. Wikit will use default meta info
                                 if this option is not provided. The template is given below(include the parentheses):
                                 (
                                     title: "A generic MDX dictionary",
                                     author: "An anonymous hero",
                                     description: "Just for fun",
                                 )
        -o, --output <output>    Same with <input>
            --table <table>      The table name in the database, you must provide this parameter if input/output is a
                                 database url

    ARGS:
        <input>    The input file format depends on the value. File suffix reflects the format: .txt => text, .mdx =>
                   mdx. If the value is a database url such as postgresql://user@localhost:5432/dictdb, then the input
                   is a database

Some usage examples

    # Create a mdx file from text source
    wikit dict --create --output /path/to/dict.mdx /path/to/dict.txt

    # Parse a mdx into text source
    wikit dict --create --output /path/to/dict.txt /path/to/dict.mdx

    # Dump information from mdx file
    wikit dict --info /path/to/dict.mdx

    # Create mac dictionary from MDX
	wikit dict --create --output test/macmillan.dictionary ~/Downloads/macmillan.mdx

    # Create mac dictionary from text(mdx soure file)
	wikit dict --create --output test/demo.dictionary test/demo.txt

Especially, when you create macos dictionary from MDX, the format used in mac is
XHTML rather than HTML. As a result, you may encounter some errors such as

       parser error : Couldn't find end of Start Tag link
       <link  href="concise_bing.css" rel="stylesheet" type="text/css"

and the link tag may look like

       <link  href="concise_bing.css" rel="stylesheet" type="text/css">

or

       <link  href="concise_bing.css" rel="stylesheet" type="text/css" //>

In the case above, the error is caused by the not self-enclosing tag `<link>`, the right content in XHTML
style is showed below

       <link  href="concise_bing.css" rel="stylesheet" type="text/css" />

To solve the problem, you can change the intermedia text file(in fact, it is the MDX source) whose name
is `${MDX_NAME}_wikit.txt` where `{MDX_NAME}` is your MDX file name. And it could be found in the same
directory of your MDX file. Let's say you have a MDX file located in `/path/to/mymdx.mdx`,
the corresponding text file will be `/path/to/mymdx_wikit.txt`. If you have trouble with XHTML,
you could change `/path/to/mymdx_wikit.txt` and run the build command again.

# Compatibility

The first-class citizens supported by Wikit are opensourced dictionary tools such as
[goldendict](https://github.com/goldendict/goldendict).  As a result, MDX created by Wikit will
mainly be tested for them. Currently, MDX is tested with goldendict version 1.5.0-RC2+git, it works
really well. If you have any other problems with the created MDX, please file an issue. MDX created
by wikit is also tested with [MDict](https://www.mdict.cn) version 2.0.12, it works but the
dictionary index seems does not work well.

# Building

- Build for your development machine

        cargo build

- Build cross-platform (mac, win, linux)

    Ensure [docker](https://www.docker.com/) is installed and then

    - make development container

            make image
            make container

    - build packages

            make publish

    The generated packages will be found in `release/` directory.

# License

[MIT](./LICENSE)
