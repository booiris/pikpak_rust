import { Configuration, RemoteListApi } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

const client = () => {
  return new RemoteListApi(
    new Configuration({
      basePath: useBackendUrlStore().storedUrl
    })
  )
}

export const remoteListApi = async () => {
  const config = {
    headers: {
      Authorization: `Bearer ${useTokenStore().storedToken}`
    }
  }
  return client().remoteList(config)
}
