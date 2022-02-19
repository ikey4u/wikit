import { invoke, convertFileSrc } from '@tauri-apps/api/tauri';

export default {
  get_dict_list: async function() {
    return await invoke('get_dict_list')
  },
  lookup: async function(dictid, word) {
    return await invoke('lookup', { dictid, word });
  },
}
