import { Configuration, DownloadResumeApi } from '@/api'
import { useBackendUrlStore } from '@/stores/backend_url'
import { useTokenStore } from '@/stores/token'

const client = new DownloadResumeApi(
    new Configuration({
        basePath: useBackendUrlStore().storedUrl,
        accessToken: useTokenStore().storedToken
    })
)

export const downloadResumeApi = async (id: string) => {
    return client.downloadResume(
        {
            file_id: id
        },
        {
            timeout: 4000
        }
    )
}
