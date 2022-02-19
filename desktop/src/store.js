import { writable } from 'svelte/store';

export const dictSettings = writable({
  'dict': {
    'all': [],
    'selected': [],
  },
});
