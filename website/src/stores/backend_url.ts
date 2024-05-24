import { ref } from "vue";
import { defineStore } from "pinia";

export const useBackendUrlStore = defineStore("backend_url", () => {
  const storedUrl = ref("");
  function setUrl(store_host: string) {
    storedUrl.value = store_host;
  }
  return { storedUrl, setUrl };
});
