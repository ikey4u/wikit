# mac 词典格式

- 源文件格式

    mac 词典源文件格式为 XHTML, 一个源文件样例如下

        <?xml version="1.0" encoding="UTF-8"?>
        <d:dictionary xmlns="http://www.w3.org/1999/xhtml" xmlns:d="http://www.apple.com/DTDs/DictionaryService-1.0.rng">
        <d:entry id="make_up_ones_mind" d:title="make up one's mind" d:parental-control="1">
            <d:index d:value="make up one's mind"/>
            <h1>make up one's mind</h1>
            <ul>
                <li>
                make a decision.
                </li>
            </ul>
        </d:entry>
        </d:dictionary>

    - XHTML 声明

        第一部分为 XHTML 头声明, 基本上不用改动

            <?xml version="1.0" encoding="UTF-8"?>

    - 词典体

        整个词典包含在一个 `<d:dictionary></d:dictionary>` 标签中, 所有词条都存放在该标签中.

        - 词条

            词条对应的格式为 `<d:entry></d:entry>`, 词条具有如下几个重要属性:

                id: 当前词条的 ID 值, 在整个词典中必须唯一
                d:title: 词条值, 一般应为当前词条的标准形式

            - 词条索引

                词条的索引为 `<d:index />`, 一个词条可以包含一个或者多个索引, 比如单词 make 具有如下
                几个索引

                    <d:index d:value="make"  d:title="make"/>
                    <d:index d:value="makes" d:title="makes"/>
                    <d:index d:value="made"  d:title="made"/>

                那么搜索 make, makes, made 都会以 make 的结果显示.

                词条索引的 d:value 属性表示搜索时触发的关键字, d:title 是搜索后显示的词条值,
                d:anchor 表示高亮词条内容的某一部分. 比如

                    <d:index d:value="make it" d:title="make it" d:parental-control="1" d:anchor="xpointer(//*[@id='make_it'])"/>

                注意词条索引必须放在词条内容之前.

            - 词条内容

                在词条索引之后就是词条内容, 可以是任意的 html.

- 参考

    - [Dictionary Services Programming Guide](https://developer.apple.com/library/archive/documentation/UserExperience/Conceptual/DictionaryServicesProgGuide/Introduction/Introduction.html#//apple_ref/doc/uid/TP40006152-CH1-SW1)