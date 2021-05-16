# Wikit - A universal dictionary

# What is it?

To be short, Wikit is a tool which can (fully, may be in future) render and create dictionary file
in MDX/MDD format.

This project is in very early stage, but you can try this [demo](http://106.53.152.194/wikit/).

# Why it?

MDX/MDD is a popular closed-source dictionary format which you can find many dictionaries made by a
variety of folks on [pdawiki](https://www.pdawiki.com) or [freedict](https://freemdict.com/).
All of this should give appreciations to the hero of `xwang` which firstly released full
reverse-engineering analysis of MDX/MDD format, which you can find
[here](https://bitbucket.org/xwang/mdict-analysis/src/master/).

Several reasons make me write this project

- I am a heavy user of dictionary since I often read english books or papers
- For what I can tell now, no project about MDX/MDD is actively maintained
- No platform-agnostic, user-friendly, fully-opensourced and free dictionary for now
- I love rust programming

# Development

Firstly, you should familiar with MDX/MDD format which can be found at
[here](https://bitbucket.org/xwang/mdict-analysis/src/master/). For MDX 2.0, you may be also interested in
the following illustraion (you can view it in fullscreen [here](https://raw.githubusercontent.com/ikey4u/wikit/dev/docs/imgs/mdx-format.svg)):
![mdx format](./docs/imgs/mdx-format.svg "mdx format")

Secondly, download a MDX/MDD dictionary from anywhere and save it to somewhere, run the following
command to do quick and dirty development

    TEST_MDX_FILE=/path/to/mdx cargo test test_parse_mdx -- --nocapture

Finally, turn your thoughts into codes and make the contributions, cool developer!

# Roadmap

- [x] MDX 2.0 parsing
- [ ] MDX 2.0 writing
- [ ] MDX 1.2 parsing and writing

# License

[MIT](./LICENSE)
