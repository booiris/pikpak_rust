<template>
    <div class="main-container">
        <el-image :src="icon" />

        <el-card shadow="always">
            <el-form
                ref="formRef"
                :model="formValue"
                :rules="rules"
                label-width="auto"
                class="login-form"
                v-loading="loadingRef"
                label-position="top"
                hide-required-asterisk
            >
                <el-text> PikPak Rust </el-text>

                <el-form-item label="Email" prop="email">
                    <el-input
                        v-model="formValue.email"
                        type="email"
                        placeholder="xxxx@xxxx"
                        autocomplete="email"
                    ></el-input>
                </el-form-item>
                <el-form-item label="Password" prop="password">
                    <el-input
                        type="password"
                        v-model="formValue.password"
                        placeholder="******************"
                        show-password
                    ></el-input>
                </el-form-item>
                <el-form-item label="Backend Addr" prop="url">
                    <el-input
                        v-model="formValue.url"
                        type="text"
                        :placeholder="defaultUrl"
                        autocomplete="on"
                    ></el-input>
                </el-form-item>
                <el-form-item>
                    <el-button type="primary" @click="login(formRef)">Login</el-button>
                </el-form-item>
            </el-form>
        </el-card>
    </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useBackendUrlStore } from '@/stores/backend_url'
import { loginApi } from '@/services/login'
import { useTokenStore } from '@/stores/token'
import axios, { AxiosError } from 'axios'
import { ElMessage, type FormInstance } from 'element-plus'
import icon from '@/assets/rust_pikpak.png'
import router from '@/router'
import { parseApiError } from '@/utils/handle_error'

const defaultUrl = `http(s)://${window.location.hostname}:22523`
const backendStoreUrl = useBackendUrlStore()
function urlValue(): string {
    if (backendStoreUrl.storedUrl != '') {
        return backendStoreUrl.storedUrl
    } else {
        return `http://${window.location.hostname}:22523`
    }
}

const formRef = ref<FormInstance>()
const formValue = reactive({
    email: import.meta.env.VITE_USERNAME,
    password: import.meta.env.VITE_PASSWORD,
    url: urlValue()
})

const loadingRef = ref(false)

const validateUrl = (rule: any, value: any, callback: any) => {
    try {
        const newUrl = new URL(value)
        if (newUrl.protocol === 'http:' || newUrl.protocol === 'https:') {
            callback()
        } else {
            callback(new Error('Invalid URL format'))
        }
    } catch (err) {
        callback(new Error('Invalid URL format'))
    }
}

const rules = {
    email: [
        {
            required: true,
            message: 'Email required',
            trigger: ['blur', 'input']
        }
    ],
    password: [
        {
            required: true,
            message: 'Password required',
            trigger: ['blur', 'input']
        }
    ],
    url: [
        {
            required: true,
            message: 'Backend Addr required',
            trigger: ['blur', 'input']
        },
        {
            validator: validateUrl,
            trigger: ['blur', 'input']
        }
    ]
}

const login = async (formEl: FormInstance | undefined) => {
    if (!formEl) {
        return
    }

    loadingRef.value = true
    await formEl.validate(async (valid) => {
        if (!valid) {
            return
        }
        backendStoreUrl.setUrl(formValue.url)
        try {
            const data = await loginApi(formValue.email, formValue.password)
            useTokenStore().setToken(data.data.token)
            ElMessage({
                message: 'Login success',
                type: 'success',
                duration: 2000
            })

            router.push({
                name: 'file_list'
            })
        } catch (e) {
            console.error('Login failed:', e)

            if (axios.isAxiosError(e)) {
                let error = parseApiError(e as AxiosError)
                ElMessage.error({
                    showClose: true,
                    message: error,
                    duration: 5000
                })
            } else {
                ElMessage.error({
                    showClose: true,
                    message: 'login failed',
                    duration: 5000
                })
            }
        }
    })

    loadingRef.value = false
}
</script>

<style scoped>
@import url('https://fonts.googleapis.com/css?family=Poppins');

.main-container {
    display: flex;
    justify-content: center;
    align-items: center;
    flex-direction: column;
    height: 85vh;
}

.el-image {
    width: 80px;
    height: 80px;

    margin-bottom: 20px;
}

.el-card {
    max-width: 640px;
    border-radius: 12px;
    border: 1px solid #d1d1d1;
    width: 400px;
    min-width: 200px;
}

.el-form {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding-inline: 20px;
    font-family: 'Poppins';
}

.el-text {
    font-size: 30px;
    text-align: center;
    margin-bottom: 10px;
}

:deep(.el-input) {
    border: none;
}

:deep(.el-input__wrapper) {
    border-radius: 12px;
    height: 40px;
    border: 5px #2b2b2b;
}

:deep(.el-button) {
    border-radius: 46px;
    height: 46px;
    font-family: 'Poppins';
    background-color: rgb(25, 25, 25);
    border: none;
    width: 100%;
    margin-top: 10px;
}
</style>
