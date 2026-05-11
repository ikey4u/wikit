# Dictionary Installation

First thing first, wikit is  merely a query software for looking up word from dictionary. It has no builtin dictionary since most dictionaries such as oxford are copyrighted.

Fortunately, with the help of project [ECDICT](https://github.com/skywind3000/ECDICT) we can make our public domain dictionaries. ECDICT is a English to Chinese dictionary which contains approximately 3, 400, 000 words.  You can get it from [here](https://github.com/ikey4u/wikit/releases/tag/dictionary-ecdit-v1.0.28).Download `ecdict.wikit.zip` from there,
unzip it to somewhere and you will get a file named `ecdict.wikit` (this is the dictionary for wikit, yes, wikit dictionary has a suffix `wikit`). Lets say you put `ecdict.wikit` to location `/path/to/ecdict.wikit`. The following steps will take this as an example to demonstrate how to install  the dictionary.

1.  Start wikit application,  click File/Configuration to open configuration directory, and you will see a file named `wikit.toml` file under the directory.

2. Open `wikit.toml` using your favorite text editor, and put the following content

    ```
    [cltcfg]
    uris = [
       "file:///path/to/ecdict.wikit",
    ]

    [srvcfg]
    uris = [
    ]
    host = "0.0.0.0"
    port = 8888

    ```

    Note the path for local dictionary must start with prefix `file://` follwed by the absolute path
    to wikit dictionary, multiple dictionaries are supported.

3. Restart wikit application and you are done

# Dictionary Creation

If you would like contribute to wikit, you could not only contribute code to the project, but also make dictionaries for it (for public or private usage)!

You have two ways to create dictionary for wikit

1. Create from source

The source file format is plain text, each word contains three parts which are showed below

    keyword
    body
    </>

The first part is the `keyword`, which is the keyword in the word search; the second part is the `body`, which is the specific html;
The third part is the terminator `</>`. `keyword` and `</>` must hold on single line but `body` can hold multiple lines.
The follwing is an example

    Whole
    <font size=5>whole</font>
    <br/>
    <font face="Kingsoft Phonetic Plain, Tahoma">(hol,hJl; houl)</font>
    </font>
    </>

2. Create from mdx dictionary

If you have already known what is mdx dictionary, you can convert mdx dictionary directly into wikit
dictionary using wikit CLI tool (a single executable named `wikit`).

To start, create a dictionary like the follwoing under any directory `somedir` and put two files
such as mydict.txt and mydict.toml where mydict is your dictionary name

    /somedir
    +-- mydict.txt
    +-- mydict.toml

        An example content of mydict.toml

            name = "简明英汉词典"
            version = "1.0.28"
            authors = ["skywind3000 <https://github.com/skywind3000>"]
            distributors = ["wikit <https://github.com/ikey4u/wikit>"]
            description = "Wikit Dictionary based on ECDICT database"
            homepage = "https://github.com/skywind3000/ECDICT/releases/tag/1.0.28"
            css = "@ecdict.css"
            js = ""

Then you can create mydict.wikit using the following command

    /path/to/wikit dict mydict.txt -c -o mydict.wikit

