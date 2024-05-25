import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useTokenStore = defineStore('jwt', () => {
  const storedToken = ref('')
  function setToken(token: string) {
    storedToken.value = token
  }
  return { storedToken, setToken }
})
