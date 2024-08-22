import { RemoteListApi } from '@/api'
import { clientWrapper } from './wrapper'

const client = clientWrapper(RemoteListApi)

export const remoteListApi = async (path: string) => {
    return client().remoteList(path, {
        timeout: 10000
    })
}
