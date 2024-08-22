import { DownloadBeginApi } from '@/api'
import { clientWrapper } from './wrapper'

const client = clientWrapper(DownloadBeginApi)

export const downloadBeginApi = async (file_id: string, output_dir: string, rename: string) => {
    return client().downloadBegin(
        { file_id: file_id, output_dir: output_dir, rename: rename },
        {
            timeout: 8000
        }
    )
}
