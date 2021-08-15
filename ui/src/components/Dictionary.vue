<template>
  <div tabindex="0" class="page">
    <div class="search-bar">
      <div class="droplist">
        <select id="active-dictionary" @change="onChangeDictionary">
          <option value="en_en_oxford_advanced" selected="selected">Oxford Advanced (En-Zh)</option>
          <option value="en_zh_cambridge">Cambridge (En-Zh)</option>
          <option value="en_zh_ltd">TLD (En-Zh)</option>
          <option value="en_zh_collins">Collins (En-Zh)</option>
          <option value="en_en_ms_bing">Bing (En-Zh)</option>

          <option value="en_en_cambridge_advanced">Cambridge Advanced (English)</option>
          <option value="en_en_collins_advanced">Collins Advanced (English)</option>
          <option value="en_en_webser_unabridged">Webster Unabridged (English)</option>
          <option value="en_en_webster_2020_online">Webster Thesaurus (English)</option>

          <option value="zh_en_new_century">新世纪汉英大词典 (Zh-En)</option>
          <option value="zh_en_ce">汉英大词典 (Zh-En)</option>
          <option value="zh_en_ce_fltrp">汉英词典 (Zh-En)</option>

          <option value="zh_extream">汉语大词典 (Chinese)</option>
        </select>
      </div>
      <input ref="user_input" id="user-input" class="search-input" @keyup.enter="click_search"/>
      <div class="search-button" @click="click_search">Search</div>
    </div>
    <div class="meaning" v-html="meaning"></div>
  </div>
</template>

<script>
import {simplized} from '../js/chinese.js';
export default {
  data: function() {
    return {
      user_input: '',
      meaning: '',
      dictionary_name: 'en_en_oxford_advanced',
      cache: {
        'en_zh_oxford': {},
        'en_zh_cambridge': {},
        'en_zh_ltd': {},
        'en_zh_collins': {},
        'en_en_cambridge_advanced': {},
        'en_en_collins_advanced': {},
        'en_en_webser_unabridged': {},
        'en_en_webster_2020_online': {},
        'en_en_ms_bing': {},
        'en_en_oxford_advanced': {},
        'zh_en_new_century': {},
        'zh_en_ce': {},
        'zh_en_ce_fltrp': {},
        'zh_extream': {},
      },
      dictstyle: {
        'en_zh_cambridge': '',
        'en_zh_ltd': 'en_zh_ltd.css',
        'en_zh_collins': '',
        'en_en_cambridge_advanced': '',
        'en_en_collins_advanced': 'en_en_collins_advanced.css',
        'en_en_webser_unabridged': 'en_en_webser_unabridged.css',
        'en_en_ms_bing': 'en_en_ms_bing.css',
        'en_en_webster_2020_online': 'en_en_webster_2020_online.css',
        'en_en_oxford_advanced': 'en_en_oxford_advanced.css',
        'zh_en_new_century': 'zh_en_new_century.css',
        'zh_en_ce': 'zh_en_ce.css',
        'zh_en_ce_fltrp': 'zh_en_ce_fltrp.css',
        'zh_extream': 'zh_extream.css',
      },
      dictscript: {
        'en_zh_cambridge': '',
        'en_zh_ltd': '',
        'en_zh_collins': '',
        'en_en_cambridge_advanced': '',
        'en_en_collins_advanced': '',
        'en_en_webser_unabridged': '',
        'en_en_ms_bing': 'en_en_ms_bing.js',
        'en_en_webster_2020_online': '',
        'en_en_oxford_advanced': 'en_en_oxford_advanced.js',
        'zh_en_new_century': '',
        'zh_en_ce': '',
        'zh_en_ce_fltrp': '',
        'zh_extream': '',
      }
    }
  },
  mounted() {
    this.$refs.user_input.focus();
    this.dictionary_name = 'en_en_oxford_advanced';
    document.getElementsByClassName("meaning")[0].onclick = this.click_word_in_meaning;
    // Enable forward and backward feature
    window.addEventListener('popstate', (event) => {
      if (event.state) {
        let state = event.state;
        this.meaning = state.meaning;
        this.update_state(state.dictionary_name, state.user_input);
      }
    });
    let page = document.getElementById("app");
    let vm = this;
    // When cursor is already foucs on input box, then after pressing the enter key, do search
    // directly.  Otherwise, make the cursor focuse into the input box.
    page.addEventListener("keyup", event => {
      let curele = document.activeElement.tagName;
      if (event.code === "Enter") {
        if (curele === "INPUT") {
          vm.click_search();
        } else {
          vm.scroll_and_focus();
        }
      }
    });
    // When the cursor is focus on the input box but the input box is not visible to the user(and user
    // does not press any key on the meaning page), if you press enter key in this situation, you
    // will not able to input anything. The settings here enables the feature that when pressing enter
    // key, go to input box and bring it into view.
    page.addEventListener("mouseover", event=> {
      page.focus({preventScroll: true});
    });
  },
  name: 'Dictionary',
  props: {
    msg: String
  },
  methods: {
    getQueryParameters() {
      let user_input = document.getElementById("user-input").value;
      let user_dict = document.getElementById("active-dictionary").value;
      return {
        'dict': user_dict,
        'word': user_input,
      };
    },
    onChangeDictionary() {
      let params = this.getQueryParameters();
      this.dictionary_name = params.dict;
      this.query(params.dict, params.word);
    },
    toggle_style(dictname) {
      let stylename = this.dictstyle[dictname];
      if (stylename && stylename.length > 0) {
        document.getElementById('dictstyle').href = `${this.$axios.defaults.baseURL}/wikit/static/${stylename}`;
      }
      // Dynamically load javascript script
      let scriptname = this.dictscript[dictname];
      if (scriptname && scriptname.length > 0) {
        let script = document.getElementById('dictscript');
        if (script) {
          script.remove();
        }
        script = document.createElement('script')
        script.src = `${this.$axios.defaults.baseURL}/wikit/static/${scriptname}`;
        script.type = 'text/javascript';
        script.id = 'dictscript';
        document.getElementsByTagName("head")[0].appendChild(script);
      }
    },
    scroll_and_focus: function() {
      window.scrollTo(0, 0);
      if (this.$refs.user_input) {
        this.$refs.user_input.focus({preventScroll: true});
      }
    },
    update_state: function(dictionary_name, user_input) {
      if (dictionary_name && dictionary_name.length > 0) {
        this.toggle_style(dictionary_name);
      }

      if (user_input && user_input.length > 0) {
        document.getElementById('user-input').value = user_input;
      }
    },
    query: function(dictionary_name, word) {
      if (!word) {
        return;
      }

      word = word.trim();
      if (word.length == 0) {
        return;
      }

      let cache_meaning = this.cache[dictionary_name][word];
      if (cache_meaning && cache_meaning.length > 0) {
        this.meaning = cache_meaning;
      } else {
        this.meaning = `Looking up word <b>${word}</b> ...`
        let vm = this;
        this.$axios.get(`/dict/q?word=${word}&dictname=${dictionary_name}`)
          .then((res) => {
            let meaning = res.data;
            // Convert Traditional Chinese to Simplified Chinese
            if (dictionary_name.startsWith('zh_')) {
              if (!meaning.startsWith('See <a href=')) {
                meaning = simplized(meaning);
              }
            }
            // This is an asyn-function, client will send request after user clicks dictionary A.
            // If the user clicks dictionary B immediately, the UI will change to page of dictionary
            // B, let say that now the request of A has finished, then the meaning of dictionary B
            // will be the meaning of dictionary A. The judgement here will avoid this mistake.
            if (this.dictionary_name == dictionary_name) {
              this.meaning = meaning;
            }
            this.cache[dictionary_name][word] = this.meaning;
            window.history.pushState(
              // state
              {
                dictionary_name: this.dictionary_name,
                meaning: this.meaning,
                user_input: this.user_input,
              },
              // title
              '',
              // url
              ''
            );
          })
        .catch(function (err) {
          let error = `Oops, some error happens: ${err}`;
          vm.meaning = error;
        });
      }
      window.scrollTo(0, 0);
      document.getElementsByClassName('meaning')[0].style.display = 'block'
      this.toggle_style(dictionary_name)
    }, // query
    click_word_in_meaning: function(event) {
      event.preventDefault();
      if (!event.target) {
        return;
      }
      let word = '';

      // Current node
      let node = event.target;
      // Parent node
      let pnode = event.target.parentElement;
      let dictname = this.dictionary_name;
      let href = node.href;

      if (href && href.startsWith("entry://") && !href.startsWith("entry://#")) {
        word = href.replace("entry://", "").trim();
      } else {
        if (dictname === "en_en_oxford_advanced") {
          if ((node.className === 'xh') || (pnode.className === "Ref")) {
            word = node.innerText;
          }
        }

        if (dictname === 'en_en_webster_2020_online') {
          if (pnode.className === 'mw_t_sx' && pnode.href) {
            word = node.innerText.trim();
          }
          if (node.tagName === 'A') {
            word = node.innerText.trim();
          }
        }
      }

      if (word && word.length > 0) {
          this.query(this.dictionary_name, word);
          this.user_input = word;
          // decodeURIComponent is used to convert URI encoded Chinese character to normal Chinese
          // character
          document.getElementById('user-input').value = decodeURIComponent(word);
      }
    },

    click_search: function (event) {
      let user_input = document.getElementById("user-input").value

      // Clear input box and meaning page when no input is given but the user clicks enter key
      if (user_input === '') {
        document.getElementsByClassName('meaning')[0].style.display = 'none';
        return;
      }

      // If the input does not change, return directly
      if (this.user_input === user_input) {
        return;
      }

      this.user_input = user_input;
      if (this.user_input && this.user_input.length > 0 && this.dictionary_name && this.dictionary_name.length > 0) {
        this.query(this.dictionary_name, this.user_input);
      }

    },

  }
}
</script>

<style scoped>

.page {
  margin: 5% 2.5% 5% 2.5%;
}

.page:focus {
  outline: 0;
}

.search-bar {
  display: flex;
  justify-content: center;
  flex-direction: row;
  width: 100%;
  border-bottom: 1px solid green;
}

.droplist select {
  border: none;
  color: black;
  padding: 2px;
  font: 12px;
}

.search-input {
  border: none;
  width: 100%;
	font: "Helvetica neue", Arial, sans-serif;
}

.search-input:focus {
  font-color: blue;
  outline: 0;
}

.search-button {
  user-select: none;
  border: none;
  color: black;
  max-width: 200px;
  height: 1.2em;
  padding: 2px;
}

.search-button:active {
  background-color: #f5f4f4;
  color: #00303f;
}

.tab-group-title {
  text-align: center;
  font-size: 20px;
  color: #2c3e50;
  margin: 10px 0px 10px 0px;
  display: none;
}

.tab-buttons {
  user-select: none;
  margin: 20px 0px 20px 0px;
  display: none;
  flex-direction: row;
  justify-content: space-between;
  width: 100%;
}

.tab-button {
  background-color: grey;
  border-radius: 10px;
  margin: 1px 2px 1px 2px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 5px 1px 5px 1px;
  color: white;
  cursor: pointer;
  font-size: 13px;
  width: 100%;
  text-align: center;
}

.tabs {
  display: flex;
  justify-content: space-around;
}

.tab-ui {
  display: none;
}

.title {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  font-size: 32px;
  color: #2c3e50;
  margin-top: 60px;
}

.meaning {
  border: 0px solid #222831;
  padding: 2% 10% 5% 10%;
  display: none;
  overflow: auto;
}
</style>
