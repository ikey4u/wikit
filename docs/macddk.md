# mac 词典构建

到 https://developer.apple.com/download/all/ 搜索 dictionary, 然后会出现 Additional Tools for
Xcode, 随便选择一个下载对应的 dmg 文件, 可以挂载也可以安装, 制作词典工具位于目录
Dictionary Development Kit 中, 我已经将该目录中的文件提取出来了, 下载地址在[这里](https://github.com/ikey4u/macddk),
这个目录的结构如下所示

    .
    ├── bin
    │   ├── add_body_record
    │   ├── add_key_index_record
    │   ├── add_reference_index_record
    │   ├── add_supplementary_key
    │   ├── build_dict.sh
    │   ├── build_key_index
    │   ├── build_reference_index
    │   ├── extract_front_matter_id.pl
    │   ├── extract_index.pl
    │   ├── extract_property.xsl
    │   ├── extract_referred_id.pl
    │   ├── generate_dict_template.sh
    │   ├── make_body.pl
    │   ├── make_dict_package
    │   ├── make_line.pl
    │   ├── make_readonly.pl
    │   ├── normalize_key_text
    │   ├── normalize_key_text.pl
    │   ├── pick_referred_entry_id.pl
    │   ├── remove_duplicate_key.pl
    │   └── replace_entryid_bodyid.pl
    ├── documents
    │   ├── Dictionary\ Development\ Kit.rtf
    │   ├── Dictionary\ Format.rtf
    │   └── DictionarySchema
    │       ├── AppleDictionarySchema.rng
    │       └── modules
    │           └── dict-struct.rng
    ├── project_templates
    │   ├── Makefile
    │   ├── MyDictionary.css
    │   ├── MyDictionary.xml
    │   ├── MyInfo.plist
    │   └── OtherResources
    │       ├── Images
    │       │   └── _internal_dictionary.png
    │       ├── MyDictionary.xsl
    │       └── MyDictionary_prefs.html
    └── samples
        ├── JapaneseDictionarySample.xml
        ├── Sample.xml
        └── SimpleSample.xml

使用该工具生成的样例词典结构如下

    /My\ Dictionary.dictionary/Contents
    ├── Body.data
    ├── DefaultStyle.css
    ├── EntryID.data
    ├── EntryID.index
    ├── Images
    │   └── _internal_dictionary.png
    ├── Info.plist
    ├── KeyText.data
    ├── KeyText.index
    ├── MyDictionary.xsl
    └── MyDictionary_prefs.html

构建词典的入口工具为 `build_dict.sh`, 其用法如下

    build_dict.sh ${BUILD_OPTS} ${DICT_NAME} ${DICT_SRC_PATH} ${DICT_CSS_PATH} ${DICT_PLIST_PATH}

其中各个选项的含义如下

    - `${BUILD_OPTS}`: 编译选项, 默认设置为空
    - `${DICT_NAME}`: 输出的词典名称
    - `${DICT_SRC_PATH}`: 输入的词典源文件, 比如 MyDictionary.xml
    - `${DICT_CSS_PATH}`: 输入的词典的 CSS 文件路径
    - `${DICT_PLIST_PATH}`: 输入的词典 plist 文件路径,比如 MyInfo.plist

该构建脚本做了如下事情

- 使用 xmllint 工具检查词典源文件, 确保没有错误

        xmllint --stream -noout "${DICT_SRC_PATH}"

- 生成 plist 文件

        plutil -s "${DICT_PLIST_PATH}"

    并将其中的 \r 替换为 \n 并保存为 dict.plist.

    使用 xsltproc 将 `bin/extract_property.xsl` 和 dict.plist 合并到 dict_prop_list.txt 中

        xsltproc "extract_property.xsl dict.plist > dict_prop_list.txt

    然后生成 customized_template.plist

        generate_dict_template.sh $COMPRESS_OPT $ENCRYPT_OPT $TRIE_OPT $IDX_DICT_VERS \
            dict_prop_list.txt > customized_template.plist

    这里的几个变量 `$COMPRESS_OPT $ENCRYPT_OPT $TRIE_OPT $IDX_DICT_VERS` 均为空.

- 词典源文件处理

    替换所有的 \r 为 \n 并存到 dict.xml 中, 然后将 dict.xml 中的所有如下模式

        <d:index> [[:blank:]\n]*
            <d:index_value> ([^<]*) </d:index_value> [[:blank:]\n]*
            <d:index_title> ([^<]*\) </d:index_title> [[:blank:]\n]*
         </d:index>

    替换为

        <d:index d:value="\1" d:title="\2"/>

    并保存到 dict_mod.xml 中, 再使用 make_line.pl 工具创建 dict.formattedSource.xml 文件

        make_line.pl dict_mod.xml > dict.formattedSource.xml

    再使用 make_body.pl 读取 dict.formattedSource.xml 并创建 dict.body 和 dict.offset 文件.

    dict.body 也是 xml 文件, dict.offset 文件内容大致如下(tab 分隔)

        dictionary_application	0	485
        make_1	485	870
        make_up_ones_mind	1355	234
        front_back_matter	1589	762

  然后提取索引

        extract_index.pl dict.formattedSource.xml > key_entry_list.txt
        extract_referred_id.pl dict.formattedSource.xml > referred_id_list.txt
        extract_front_matter_id.pl ${DICT_PLIST_PATH} >> referred_id_list.txt

- 创建词典

    从 customized_template.plist 初始化词典 dict.dictionary

        make_dict_package dict.dictionary customized_template.plist

    处理 dict.offsets 和 dict.body, 向 dict.dictionary 添加 Body.data 文件数据

        add_body_record dict.dictionary Body.data dict.offsets dict.body

    add_body_record 的输出保存到 entry_body_list.txt.

    创建索引数据 key_body_list

        replace_entryid_bodyid.pl entry_body_list.txt < key_entry_list.txt > key_body_list.txt 

    正则化处理

        normalize_key_text < key_body_list.txt > normalized_key_body_list_1.txt
        add_supplementary_key < normalized_key_body_list_1.txt > normalized_key_body_list_2.txt

    去重复

        remove_duplicate_key.pl < normalized_key_body_list_2.txt > normalized_key_body_list.txt

    添加 KeyText.index 文件

        build_key_index dict.dictionary KeyText.index normalized_key_body_list.txt 10.5

    创建 reference 索引

        pick_referred_entry_id.pl referred_id_list.txt < entry_body_list.txt > referred_entry_body_list.txt

    添加 EntryID.index 文件
    
        build_reference_index dict.dictionary EntryID.index referred_entry_body_list.txt
