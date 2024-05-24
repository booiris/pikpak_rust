import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useJWTStore = defineStore('jwt', () => {
  const storedJWT = ref('')
  function setJWT(JWT: string) {
    storedJWT.value = JWT
  }
  return { storedJWT, setJWT }
})
