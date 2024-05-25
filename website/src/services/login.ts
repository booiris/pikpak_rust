import { Configuration, LoginApi } from '@/api'
import type { LoginReq } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'

const client = () => {
  return new LoginApi(new Configuration({ basePath: useBackendUrlStore().storedUrl }))
}

export const loginApi = async (email: string, password: string) => {
  const req = { email, password } as LoginReq
  return client().login(req)
}
