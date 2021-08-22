# Wikit - A universal dictionary

# What is it?

To be short, Wikit is a tool which can (fully, may be in future) render and create dictionary file
in MDX/MDD format.

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

    wikit 0.1.0
    ikey4u <pwnkeeper@gmail.com>
    A universal dictionary - Wikit

    USAGE:
        wikit [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        help      Prints this message or the help of the given subcommand(s)
        mdx       Process MDX file
        server    Run wikit as an API server

There are serveral subcommands: `mdx` and `server`. However `server` is not stable for now,
please use it carefully. For `mdx` subcommand, you can print its help information using following command

    wikit mdx

An example output is showed below

    Process MDX file

    USAGE:
        wikit mdx [FLAGS] [OPTIONS] <input>

    FLAGS:
        -c, --create     Create mdx file
        -h, --help       Prints help information
            --info       Dump basic information of mdx file
            --meta       You could specify a meta file when create mdx file. Wikit will use default meta info if this option
                         is not provided. The template is given below(include the parentheses):
                         (
                             title: "A generic MDX dictionary",
                             author: "An anonymous hero",
                             description: "Just for fun",
                         )
        -p, --parse      Parse mdx file
        -V, --version    Prints version information

    OPTIONS:
        -o, --output <output>    Same with <input>
            --table <table>      The table name in the database, you must provide this parameter if input/output is a
                                 database url

    ARGS:
        <input>    The input file format depends on the value. If the value has a suffix .txt, then the input is a txt
                   file; If the value has a suffix .mdx, then the input is a mdx file; If the value is a database url
                   such as postgresql://user@localhost:5432/dictdb, then the input is a database

Some usage examples

    # Create a mdx file from text source
    wikit mdx --create --output /path/to/dict.mdx /path/to/dict.txt

    # Parse a mdx into text source
    wikit mdx --parse --output /path/to/dict.txt /path/to/dict.mdx

    # Dump information from mdx file
    wikit mdx --info /path/to/dict.mdx

# Compatibility

The first-class citizens supported by Wikit are opensourced dictionary tools such as
[goldendict](https://github.com/goldendict/goldendict).  As a result, MDX created by Wikit will
mainly be tested for them. Currently, MDX is tested with goldendict version 1.5.0-RC2+git, it works
really well. If you have any other problems with the created MDX, please file an issue. MDX created
by wikit is also tested with [MDict](https://www.mdict.cn) version 2.0.12, it works but the
dictionary index seems does not work well.

# Building

- Build only for your development machine

        cargo build

- Build cross-platform (mac, win, linux)

    - Install [docker](https://www.docker.com/)
    - Create development image

            docker build -t wikitdev:v0.0.1 --progress plain --no-cache -f Dockerfile  .

        This command takes about 20 minutes to finish if you have a not-so-bad network.

    - Create development container

            docker run --name wikit -dit -v $(git rev-parse --show-toplevel):/wikitdev/wikit wikitdev:v0.0.1 bash
            docker start wikit

    - Run build

            docker exec -it wikit bash /wikitdev/wikit/scripts/build_on_docker.sh

        or just run

            make publish

        The generated packages will be found in `release/` directory.

# Development

Firstly, you should familiar with MDX format which is showed in the following illustraion (you can
view it in fullscreen [here](https://raw.githubusercontent.com/ikey4u/wikit/master/docs/imgs/mdx-format.svg)):
![mdx format](./docs/imgs/mdx-format.svg "mdx format")

Secondly, download a MDX dictionary from anywhere and save it to somewhere, run the following
command to do quick and dirty development

    # create MDX file
    make test-create
    # parse MDX file
    make test-parse

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
- [ ] Integrated testing
- [ ] Create desktop client for Linux, Mac and Windows
- [ ] MDD parsing
- [ ] GUI tools to create, parse and modify MDX
- [ ] Create mobile client for Android and iOS

# License

[MIT](./LICENSE)