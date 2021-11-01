<script>
  import { onMount } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/tauri'

  let timer;
  let default_content = `
    <div style="color: grey; display: flex; flex-direction: column; justify-content: flex-start; align-items: center; height: 300px; width: 100%; font-size: 18px;">
    <p> Type a word to look up in ...  </p>
    <p> Powered by <a target="_blank" href="https://github.com/ikey4u/wikit">wikit</a> </p>
    </div>`;
  let content = default_content;

  let input;
  let dictlist;
  onMount(() => {
    invoke('get_dict_list').then((r) => {
      console.log(`dictlist: ${JSON.stringify(r, null, 4)}`);
      dictlist = r;
    });
  })

  function lookup(input) {
    if (!input || input.lenght <= 0 || input.trim().length <= 0) {
      content = default_content;
      return;
    }

    if (!dictlist || dictlist.lenght <= 0) {
      alert("please select at least one dictionary");
      return;
    }

    input = input.trim();

    invoke('lookup', {
      dictpath: dictlist[0],
      word: input,
    }).then((meanings) => {
      if (meanings[input]) {
        content = meanings[input];
      } else {
        let possible_words = [];
        for (let key in meanings) {
          possible_words.push(key);
        }
        if (possible_words.length > 0) {
          content = `<p> not found <b>${input}</b>, would you mean <b>${possible_words}</b>? </p>`;
        } else {
          content = `<p> not found <b>${input}</b> and related words </p>`
        }
      }
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
    <div class="button options-button">+</div>
    <div class="search">
      <input class="search-input" type="text" on:keyup={({ target: { value } }) => debounce(value)} />
      <div class="button search-button" on:click={lookup}>GO</div>
    </div>
  </div>
  <div class="page">
    <div class="page-content">
      {@html content}
    </div>
  </div>
</main>

<style>
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
