import { Configuration, Filter, MgetDownloadStatusApi } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

const client = new MgetDownloadStatusApi(
    new Configuration({
        basePath: useBackendUrlStore().storedUrl,
        accessToken: useTokenStore().storedToken
    })
)

export const mgetDownloadStatusApi = async (filter?: Filter) => {
    return client.mgetDownloadStatus(filter, {
        timeout: 2000
    })
}
