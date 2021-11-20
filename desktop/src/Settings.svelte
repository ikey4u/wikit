 <Dialog
  bind:open
  selection
  aria-labelledby="list-selection-title"
  aria-describedby="list-selection-content"
  on:SMUIDialog:closed={closeHandler}
>
  <Title id="list-selection-title">Select Dictionary</Title>
  {#if $dictSettings.dict.all.length > 0}
    <Content id="list-selection-content">
      <List radioList>
        {#each $dictSettings.dict.all as dict, i}
          <Item>
            <Graphic>
              <Radio bind:group={selection} value="{dict}" />
            </Graphic>
            <Text>{dict}</Text>
          </Item>
        {/each}
      </List>
    </Content>
  {:else}
    <p> no dictionary is added </p>
  {/if}
  <Actions>
    <Button>
      <Label>CANCEL</Label>
    </Button>
    <Button action="accept">
      <Label>OK</Label>
    </Button>
  </Actions>
</Dialog>

<div class="button options-button" on:click={ () => (open = true) }>+</div>

<script lang="ts">
  import Dialog, { Title, Content, Actions, InitialFocus } from '@smui/dialog';
  import Button, { Label } from '@smui/button';
  import List, { Item, Graphic, Text } from '@smui/list';
  import Radio from '@smui/radio';
  import { dictSettings } from './store.js';

  let selection = '';
  let open = false;

  function closeHandler(e) {
    if (e.detail.action === 'accept') {
      if (selection.trim().length > 0) {
        $dictSettings.dict.selected = [selection];
      }
    }
  }
</script>

<style>
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
</style>
