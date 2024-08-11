<template>
    <div class="main">
        <el-container class="main-container">
            <el-header style="margin: 10px 5px">
                <MainHeader />
            </el-header>
            <el-container>
                <el-aside width="200px">
                    <MainAside />
                </el-aside>

                <el-main>
                    <router-view v-slot="{ Component }">
                        <keep-alive include="FileList">
                            <component :is="Component" />
                        </keep-alive>
                    </router-view>
                </el-main>
            </el-container>
        </el-container>
    </div>
</template>

<script setup lang="ts">
import MainAside from '@/components/MainAside.vue'
import MainHeader from '@/components/MainHeader.vue'
</script>

<style scoped>
.main {
    height: 100vh;
}
</style>
