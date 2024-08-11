<template>
    <el-menu default-active="file_list" router>
        <el-menu-item index="file_list">
            <el-icon><Files /></el-icon>
            <template #title>All Files</template>
        </el-menu-item>
        <el-menu-item index="file_downloading">
            <el-icon><Download /></el-icon>
            <template #title>Downloading</template>
        </el-menu-item>
        <el-menu-item index="file_downloaded">
            <el-icon><DocumentChecked /></el-icon>
            <template #title>Downloaded</template>
        </el-menu-item>
    </el-menu>
</template>

<script setup lang="ts">
import { Files, Download, DocumentChecked } from '@element-plus/icons-vue'
</script>

<style scoped></style>
