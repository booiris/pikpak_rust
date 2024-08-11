import { Configuration, DownloadRemoveApi } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

const client = new DownloadRemoveApi(
    new Configuration({
        basePath: useBackendUrlStore().storedUrl,
        accessToken: useTokenStore().storedToken
    })
)

export const downloadRemoveApi = async (file_id: string, need_remove_file: boolean) => {
    return client.downloadRemove(
        { file_id: file_id, need_remove_file: need_remove_file },
        {
            timeout: 8000
        }
    )
}
