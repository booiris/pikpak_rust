import { Configuration, RemoteListApi } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

const client = new RemoteListApi(
    new Configuration({
        basePath: useBackendUrlStore().storedUrl,
        accessToken: useTokenStore().storedToken
    })
)

export const remoteListApi = async (path: string) => {
    return client.remoteList(path, {
        timeout: 10000
    })
}
