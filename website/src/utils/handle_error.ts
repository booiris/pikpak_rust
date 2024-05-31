import { instanceOfBaseResp, type BaseResp } from '@/api/models'
import type { AxiosError } from 'axios'

export function parseApiError(error: AxiosError): string {
    const data = error.response?.data
    let serverError
    if (data instanceof Object && instanceOfBaseResp(error.response?.data as object)) {
        serverError = (error.response?.data as BaseResp).message
    } else {
        serverError = error.message
    }
    return serverError
}
