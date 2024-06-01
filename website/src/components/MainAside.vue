<template>
    <el-radio-group v-model="isCollapse" style="margin-bottom: 20px">
        <el-radio-button :value="false">expand</el-radio-button>
        <el-radio-button :value="true">collapse</el-radio-button>
    </el-radio-group>

    <el-menu default-active="file_list" :collapse="isCollapse" router>
        <el-menu-item index="file_list">
            <el-icon><Files /></el-icon>
            <template #title>All Files</template>
        </el-menu-item>
        <el-menu-item index="file_downloading">
            <el-icon><Download /></el-icon>
            <template #title>Downloading</template>
        </el-menu-item>
    </el-menu>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Files, Download } from '@element-plus/icons-vue'

const isCollapse = ref(false)
</script>

<style scoped>
.el-menu-vertical-demo:not(.el-menu--collapse) {
    width: 200px;
    min-height: 400px;
}
</style>
