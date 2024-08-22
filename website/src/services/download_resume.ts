import { DownloadResumeApi } from '@/api'
import { clientWrapper } from './wrapper'

const client = clientWrapper(DownloadResumeApi)

export const downloadResumeApi = async (id: string) => {
    return client().downloadResume(
        {
            file_id: id
        },
        {
            timeout: 4000
        }
    )
}
