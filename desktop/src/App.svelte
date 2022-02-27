<script>
  import { onMount, onDestroy } from 'svelte';
  import { listen, emit } from "@tauri-apps/api/event";
  import Settings from './Settings.svelte';
  import { dictSettings } from './store.js';
  import ffi from './ffi.js';

  let timer;
  let input;
  let default_content = `
    <div style="color: grey; display: flex; flex-direction: column; justify-content: flex-start; align-items: center; height: 300px; width: 100%; font-size: 18px;">
    <p> Type a word to look up in ...  </p>
    <p> Powered by <a target="_blank" href="https://github.com/ikey4u/wikit">wikit</a> </p>
    </div>`;
  let content = default_content;
  let isSettingsOpened = false;
  let unlisten;
  onMount(async () => {
    let r = await ffi.get_dict_list();
    $dictSettings.dict.all = r;
    if (r.length > 0) {
      $dictSettings.dict.selected = [r[0].id];
    }

    unlisten = listen("rust-event", (e) => {
      console.log("got rust event: " + e);
    });
  })

  onDestroy(() => {
    if (unlisten) {
      unlisten()
    }
  })

  function emitEvent() {
    emit("js-event", "this is the payload string");
  }

  async function lookup(input) {
    if (!input || input.length <= 0 || input.trim().length <= 0) {
      content = default_content;
      return;
    }

    if ($dictSettings.dict.selected.length <= 0) {
      alert("please add and select at least one dictionary");
      return;
    }

    let dictid = $dictSettings.dict.selected[0];
    if (!dictid) {
      if ($dictSettings.dict.all.length > 0) {
        dictid = $dictSettings.dict.all[0].id
      } else {
        console.log("no dictionary is found, stop lookup");
        return;
      }
    }

    input = input.trim().toLowerCase();
    let resp = await ffi.lookup(dictid, input);
    let meanings = resp["words"];
    let meaning = meanings[input]
    if (!meaning) {
      let possible_words = [];
      for (let key in meanings) {
        possible_words.push(key);
      }
      if (possible_words.length > 0) {
        meaning = `<p> not found <b>${input}</b>, would you mean <b>${possible_words}</b>? </p>`;
      } else {
        meaning = `<p> not found <b>${input}</b> and related words </p>`
      }
    }
    updateContent(meaning, resp["script"], resp["style"]);
  }

  function openSettings() {
    isSettingsOpened = true;
  }

  function updateContent(value, script, style) {
    let page = `
      <!DOCTYPE html>
      <html>
        <head>
          <meta charset="UTF-8" />
          ${script}
          ${style}
        </head>
        <body>
          ${value}
        </body>
      </html>
    `;
    content = page;
  }

  let frame;
  function onFrameLoad() {
    // Remove all inline onclick listeners, since its not allowed by content security policy (CSP).
    // But CSP allows us use addEventListener to listen click event.
    const inlineClicks = frame.contentDocument.querySelectorAll('[onclick]');
    inlineClicks.forEach(element => {
      let funcname = element.getAttribute("onclick").split('(')[0].trim();
      if (funcname) {
        element.addEventListener('click', (e) => {
          if (typeof frame.contentWindow[funcname] === "function") {
            frame.contentWindow[funcname](e.target);
          }
        });
      }
      element.removeAttribute('onclick');
    });
  }

 	const debounce = v => {
		clearTimeout(timer);
		timer = setTimeout(() => {
			input = v;
		}, 750);
	};

  $: {
    lookup(input);
  }
</script>

<main>
  <div class="header">
    <Settings />
    <div class="search">
      <input class="search-input" type="text" on:keyup={({ target: { value } }) => debounce(value)} />
      <div class="button search-button" on:click={lookup}>GO</div>
    </div>
  </div>
  <div class="page">
    <div class="page-content">
      <!-- use iframe to separate css and script from main program -->
      <iframe title="dictview" srcdoc={content} bind:this={frame} on:load={onFrameLoad}></iframe>
    </div>
  </div>
</main>

<style>
  iframe {
    border: none;
    height: 98%;
    width: 98%;
  }

  main {
    width: 100%;
    height: 100%;

    display: flex;
    flex-direction: column;
    flex-wrap: nowrap;
  }

  .header {
    width: 100%;
    display: flex;
    flex-direction: row;
    align-items: center;
  }

  .button {
    user-select: none;
    cursor: pointer;
  }

  .options-button {
    margin-left: 20px;
    margin-right: 40px;
    font-size: 30px;
    height: 40px;
    width: 40px;
    text-align: center;
    background: blue;
    color: white;
    border-radius: 50%;
  }

  .search {
    display: flex;
    flex-direction: row;
    align-items: center;
    flex: 1;
  }

  .search-input {
    flex: 1;
    margin: auto;
  }

  .search-button {
    margin-left: 10px;
    vertical-align: middle;
    text-align: center;
    color: #444;
    border: 1px solid #CCC;
    background: #DDD;
    max-width: 100px;
    padding: 5px;
  }

  .page {
    display: flex;
    flex-direction: column;
    /* hold reamin area */
    flex-grow: 1;
    /*
      making page left padding zero will let the appearance of scroll bar more natural in desktop
      application
    */
    padding: 20px 0px 10px 30px;
    /* scoll only page div */
    overflow: auto;
  }

  .page-content {
    flex-grow: 1;
    overflow: auto;
    min-height: 2em;
    /* wrap line if it is too long to overflow x axis */
    overflow-wrap: break-word;
  }
</style>
