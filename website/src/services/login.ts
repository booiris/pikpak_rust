import axios from 'axios'
import { useBackendUrlStore } from '@/stores/backend_url'

const axiosClient = axios.create({})

axiosClient.interceptors.request.use((config) => {
  const urlStore = useBackendUrlStore()
  config.baseURL = urlStore.storedUrl
  // config.headers.set('Access-Control-Allow-Origin', process.env.VUE_APP_Access_Control_Allow_Origin)
  return config
})

export async function login(email: string, password: string): Promise<string> {
  try {
    const response = await axiosClient.post(`/api/login`, {
      email: email,
      password: password
    })
    return response.data
  } catch (error: any) {
    if (error.response) {
      // 请求成功发出且服务器也响应了状态码，但状态代码超出了 2xx 的范围
      console.error('Error response:', error.response.data)
      throw error.response.data
    } else if (error.request) {
      // 请求已经成功发起，但没有收到响应
      console.error('Error request:', error.request)
      throw new Error('No response received from server')
    } else {
      // 发送请求时出了点问题
      console.error('Error message:', error.message)
      throw new Error('Error in request setup')
    }
  }
}
