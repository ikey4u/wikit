import { writable } from 'svelte/store';

export const dictSettings = writable({
  'dict': {
    // array of dictionary, element format is `{ "id:: "<the dict id>", "name": "dictionary name for human" }`
    'all': [],
    // selected dictionary ID
    'selected': [],
  },
});
