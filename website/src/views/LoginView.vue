<template>
    <n-card title="Login" style="width: 400px; margin: auto; margin-top: 100px;">
        <n-form @submit.prevent="handleLogin">
            <n-form-item label="Username" path="username">
                <n-input v-model="username" placeholder="Enter your username" />
            </n-form-item>
            <n-form-item label="Password" path="password">
                <n-input type="password" v-model="password" placeholder="Enter your password" />
            </n-form-item>
            <n-form-item>
                <n-button type="primary" native-type="submit">Login</n-button>
            </n-form-item>
        </n-form>
    </n-card>
</template>

<script setup lang="ts">
</script>

<style scoped></style>