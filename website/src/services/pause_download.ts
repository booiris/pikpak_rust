import { Configuration, DownloadPauseApi } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

const client = new DownloadPauseApi(
    new Configuration({
        basePath: useBackendUrlStore().storedUrl,
        accessToken: useTokenStore().storedToken
    })
)

export const pauseDownloadApi = async (id: string) => {
    return client.downloadPause(
        {
            file_id: id
        },
        {
            timeout: 4000
        }
    )
}
