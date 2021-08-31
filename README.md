# Wikit - A universal dictionary

# What is it?

To be short, Wikit is a tool which can (fully, may be in future) create or parse dictionary file
in MDX/MDD/Mac Dictionary format.

This project is in very early stage, but you can try this [demo](http://106.53.152.194/wikit/)
online or download electron wrapped client from here
[Releases](https://github.com/ikey4u/wikit/releases).

# Why it?

MDX/MDD is a popular closed-source dictionary format which you can find many dictionaries made by a
variety of folks on [pdawiki](https://www.pdawiki.com) or [freemdict](https://freemdict.com/).
All of these should give appreciations to the hero of `xwang` who firstly released full
reverse-engineering analysis of MDX/MDD format, which you can find
[here](https://bitbucket.org/xwang/mdict-analysis/src/master/).

Several reasons make me write this project

- I am a heavy user of dictionary since I often read english books or papers
- For what I can tell now, no project about MDX/MDD is actively maintained
- No platform-agnostic, user-friendly, fully-opensourced and free dictionary for now
- I love rust programming

# Installation and Usage

There is no GUI but only CLI for Wikit for now, you can download the tool from [release](https://github.com/ikey4u/wikit/releases) page.

Decompress the release packege and just fireup the tool `wikit`, you will see detail help information,
for example

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

# Development

Firstly, you should familiar with MDX format which is showed in the following illustraion (you can
view it in fullscreen [here](https://raw.githubusercontent.com/ikey4u/wikit/master/docs/imgs/mdx-format.svg)):
![mdx format](./docs/imgs/mdx-format.svg "mdx format")

Secondly, download a MDX dictionary from anywhere and save it to somewhere, to process the mdx file,
see `makefile` for detail.

Finally, turn your thoughts into codes and make the contributions, cool developer!

# Credits

- [An Analysis of MDX/MDD File Format](https://bitbucket.org/xwang/mdict-analysis/src/master/) by [xwang](https://bitbucket.org/xwang)

    The first attempt to analysis MDX/MDD file format.

- [writemdict](https://github.com/zhansliu/writemdict) by [zhansliu](https://github.com/zhansliu)
  
    A python library to generate MDX file and give a detail description of MDX format.

# Roadmap

- [x] MDX 1.2 parsing
- [x] MDX 2.0 parsing
- [x] MDX 2.0 writing
- [ ] Create mac dictionary (WIP)
- [ ] MDD parsing
- [ ] Create desktop client for Linux, Mac and Windows
- [ ] GUI tools to create, parse and modify MDX
- [ ] Create mobile client for Android and iOS

# License

[MIT](./LICENSE)
