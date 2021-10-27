<script>
  import { onMount } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/tauri'

  let content = `
    <div style="color: grey; display: flex; flex-direction: column; justify-content: flex-start; align-items: center; height: 300px; width: 100%; font-size: 18px;">
    <p> Type a word to look up in ...  </p>
    <p> Powered by <a target="_blank" href="https://github.com/ikey4u/wikit">wikit</a> </p>
    </div>`;
  let input;
  let dictlist;
  onMount(() => {
      invoke('get_dict_list').then((r) => {
        console.log(`dictlist: ${JSON.stringify(r, null, 4)}`);
        dictlist = r;
      });
  })
  function lookup() {
      console.log(input);
  }
</script>

<main>
  <div class="header">
    <div class="button options-button">+</div>
    <div class="search">
      <input class="search-input" type="text" bind:value={input} />
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
    flex-grow: 1;
    padding: 20px 30px 10px 30px;
  }

  .page-content {
    flex-grow: 1;
    overflow: auto;
    min-height: 2em;
  }
</style>
