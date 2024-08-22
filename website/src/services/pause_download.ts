import { DownloadPauseApi } from '@/api'
import { clientWrapper } from './wrapper'

const client = clientWrapper(DownloadPauseApi)

export const pauseDownloadApi = async (id: string) => {
    return client().downloadPause(
        {
            file_id: id
        },
        {
            timeout: 4000
        }
    )
}
