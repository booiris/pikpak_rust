import { Filter, MgetDownloadStatusApi } from '@/api'
import { clientWrapper } from './wrapper'

const client = clientWrapper(MgetDownloadStatusApi)

export const mgetDownloadStatusApi = async (filter?: Filter[]) => {
    return client().mgetDownloadStatus(
        {
            filter
        },
        {
            timeout: 2000
        }
    )
}
