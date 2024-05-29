<template>
    <div class="login-container">
        <div class="login-box">
            <div class="logo-container">
                <img src="@/assets/rust_pikpak.png" alt="logo" class="logo" />
            </div>
            <h2>RUST PIKPAK</h2>
            <form @submit.prevent="login">
                <div :class="['input-group', { error: errors.email }]">
                    <label for="email">Email</label>
                    <input type="email" id="email" v-model="email" placeholder="xxx@gmail.com"
                        :class="{ 'input-error': errors.email }" />
                    <span v-if="errors.email" class="error-message">{{ errors.email }}</span>
                </div>
                <div :class="['input-group', { error: errors.password }]">
                    <label for="password">Password</label>
                    <div class="password-wrapper">
                        <input :type="passwordVisible ? 'text' : 'password'" id="password" v-model="password"
                            placeholder="********" :class="{ 'input-error': errors.password }" />
                        <button type="button" class="toggle-password" @click="togglePasswordVisibility">
                            <!-- <span v-show="passwordVisible"><i class="fa-regular fa-eye" /></span>
                            <span v-show="!passwordVisible"><i class="fa-regular fa-eye-slash" /></span> -->
                        </button>
                    </div>
                    <span v-if="errors.password" class="error-message">{{ errors.password }}</span>
                </div>
                <div :class="['input-group', { error: errors.url }]">
                    <label for="url">Backend Addr</label>
                    <input type="text" id="url" v-model="url" :placeholder="hintUrl()"
                        :class="{ 'input-error': errors.url }" />
                    <span v-if="errors.url" class="error-message">{{ errors.url }}</span>
                </div>
                <button type="submit" class="login-button">Submit</button>
            </form>
        </div>
        <LoginModal :show="showModal" @close="showModal = false">
            <template v-slot:header>
                <h3>错误</h3>
            </template>
            <template v-slot:body>
                <p>{{ serverError }}</p>
            </template>
            <template v-slot:footer>
                <button @click="showModal = false">关闭</button>
            </template>
        </LoginModal>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useBackendUrlStore } from '@/stores/backend_url';
import { useTokenStore } from '@/stores/token';
import { loginApi } from '@/services/login';
import { remoteListApi } from '@/services/remote_list';
import LoginModal from '@/components/LoginModal.vue'
import type { AxiosError } from 'axios';
import type { BaseResp } from '@/api';
import axios from 'axios';
import { instanceOfBaseResp } from '@/api/models';

const email = ref<string>('');
const password = ref<string>('');
const url = ref<string>(`http://${window.location.hostname}:22522`);
const passwordVisible = ref<boolean>(false);
const errors = ref<{ email: string, password: string, url: string }>({
    email: '',
    password: '',
    url: ''
});
const showModal = ref<boolean>(false);
const serverError = ref<string>('');

const login = async () => {
    errors.value = { email: '', password: '', url: '' };

    if (!email.value) {
        errors.value.email = 'Email required';
    }

    if (!password.value) {
        errors.value.password = 'Password required';
    }

    if (!url.value) {
        errors.value.url = 'URL required';
    } else if (!validateUrl(url.value)) {
        errors.value.url = 'Invalid URL format';
    }

    if (!errors.value.email && !errors.value.password && !errors.value.url) {
        useBackendUrlStore().setUrl(url.value);
        try {
            const data = await loginApi(email.value, password.value);
            console.log('Login successful:', data);
            useTokenStore().setToken(data.data.token);
            await remoteListApi("/");
        } catch (e) {
            if (axios.isAxiosError(e)) {
                let error = e as AxiosError;
                console.error('Login failed:', error);
                let data = error.response?.data
                if (data instanceof Object && instanceOfBaseResp(error.response?.data as object)) {
                    serverError.value = (error.response?.data as BaseResp).message;
                } else {
                    serverError.value = JSON.stringify(e)
                }
                showModal.value = true;
            } else {
                console.error('Login failed:', e);
            }
        }
    }
};

const togglePasswordVisibility = () => {
    passwordVisible.value = !passwordVisible.value;
};

const validateUrl = (url: string): boolean => {
    try {
        const newUrl = new URL(url);
        return newUrl.protocol === 'http:' || newUrl.protocol === 'https:';
    } catch (err) {
        return false;
    }
};

const hintUrl = () => {
    return `http(s)://${window.location.hostname}:22522`;
};
</script>

<style scoped>
body {
    margin: 0;
    font-family: Arial, sans-serif;
}

.login-container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    background: linear-gradient(135deg, #72edf2 10%, #5151e5 100%);
    padding: 20px;
}

.login-box {
    background: #ffffff;
    padding: 40px 30px;
    border-radius: 10px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
    text-align: center;
    max-width: 450px;
    width: 100%;
}

.logo-container {
    display: flex;
    justify-content: center;
    margin-bottom: 20px;
}

.logo {
    max-width: 100px;
}

h2 {
    margin-bottom: 30px;
    color: #333;
    font-weight: 500;
    font-family: 'Arial', sans-serif;
}

.input-group {
    margin-bottom: 20px;
    position: relative;
    text-align: left;
}

label {
    display: block;
    margin-bottom: 5px;
    color: #606266;
}

.password-wrapper {
    position: relative;
    width: 100%;
}

input {
    width: 100%;
    padding: 12px 15px;
    border: 1px solid #dcdfe6;
    border-radius: 8px;
    font-size: 16px;
    outline: none;
    transition:
        border-color 0.3s ease,
        box-shadow 0.3s ease;
}

input:focus {
    border-color: #409eff;
    box-shadow: 0 0 5px rgba(64, 158, 255, 0.5);
}

.input-error {
    border-color: #f56c6c;
}

.error-message {
    color: #f56c6c;
    font-size: 14px;
    position: absolute;
    bottom: -20px;
    left: 0;
}

.toggle-password {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
}

.login-button {
    width: 100%;
    padding: 12px 15px;
    background-color: #409eff;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    cursor: pointer;
    transition:
        background-color 0.3s ease,
        transform 0.3s ease;
}

.login-button:hover {
    background-color: #66b1ff;
    transform: translateY(-2px);
}

@media (max-width: 768px) {
    .login-box {
        padding: 20px 15px;
        box-shadow: none;
    }

    .logo {
        max-width: 80px;
    }

    h2 {
        font-size: 18px;
    }

    .input-group {
        margin-bottom: 15px;
    }

    label {
        font-size: 14px;
    }

    input {
        padding: 10px 12px;
        font-size: 14px;
    }

    .login-button {
        padding: 10px 12px;
        font-size: 14px;
    }

    .error-message {
        font-size: 12px;
        bottom: -15px;
    }

    .toggle-password {
        font-size: 14px;
    }
}
</style>
