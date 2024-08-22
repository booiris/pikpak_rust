import { DownloadRemoveApi } from '@/api'
import { clientWrapper } from './wrapper'

const client = clientWrapper(DownloadRemoveApi)

export const downloadRemoveApi = async (file_id: string, need_remove_file: boolean) => {
    return client().downloadRemove(
        { file_id: file_id, need_remove_file: need_remove_file },
        {
            timeout: 8000
        }
    )
}
