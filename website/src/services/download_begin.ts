import { Configuration, DownloadBeginApi } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

const client = new DownloadBeginApi(
    new Configuration({
        basePath: useBackendUrlStore().storedUrl,
        accessToken: useTokenStore().storedToken
    })
)

export const downloadBeginApi = async (file_id: string) => {
    return client.downloadBegin(
        { file_id: file_id },
        {
            timeout: 2000
        }
    )
}
