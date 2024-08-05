import { Configuration, DownloadBeginApi } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

const client = new DownloadBeginApi(
    new Configuration({
        basePath: useBackendUrlStore().storedUrl,
        accessToken: useTokenStore().storedToken
    })
)

export const downloadBeginApi = async (file_id: string, output_dir: string, rename: string) => {
    return client.downloadBegin(
        { file_id: file_id, output_dir: output_dir, rename: rename },
        {
            timeout: 2000
        }
    )
}
