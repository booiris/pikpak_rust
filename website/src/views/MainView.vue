<template>
    <div class="main">
        <el-container class="main-container">
            <el-header class="main-header">
                <el-row :gutter="20">
                    <el-col :span="6">
                        <div class="icon-display">
                            <el-image :src="icon" class="icon-img" />
                            <el-text class="icon-title">PikPak Rust</el-text>
                        </div>
                    </el-col>
                    <el-col :span="18">
                        <div class="file-list" style="background-color: grey; height: 100%" />
                    </el-col>
                </el-row>
            </el-header>
            <el-container class="submain-container">
                <el-aside class="main-aside" width="200px"
                    style="border: 1px solid grey; height: 700px">Aside</el-aside>
                <el-main class="main-main">
                    <el-table :data="tableData" style="width: 100%">
                        <el-table-column prop="fileName" label="Name" width="500" />
                        <el-table-column prop="size" label="Size" width="180" />
                        <el-table-column prop="updateTime" label="Update Time" />
                    </el-table>
                </el-main>
            </el-container>
        </el-container>
    </div>
</template>

<script setup lang="ts">
import icon from '@/assets/rust_pikpak.png'
import { remoteListApi } from '@/services/remote_list'
import { parseApiError } from '@/utils/handle_error'
import type { AxiosError } from 'axios'
import { ElMessage } from 'element-plus'
import { onMounted, reactive } from 'vue'
import { parseISO, format } from 'date-fns';
import prettyBytes from 'pretty-bytes';

interface tableDataType {
    fileName: string
    size: string
    updateTime: string
}

const tableData = reactive<Array<tableDataType>>([])

onMounted(async () => {
    const data = await remoteListApi('/').catch((err: AxiosError) => {
        console.error(err)
        let error = parseApiError(err)
        ElMessage.error({
            showClose: true,
            message: 'get remote file list failed, err: ' + error,
            duration: 5000
        })
    })
    if (!data) {
        return
    }
    console.log(data)
    for (let file_status of data.data.files_info) {
        const date = parseISO(file_status.modified_time);
        let size = "-"
        if (file_status.kind != "drive#folder") {
            size = prettyBytes(Number(file_status.size))
        }
        tableData.push({
            fileName: file_status.name,
            size: size,
            updateTime: format(date, 'yyyy-MM-dd HH:mm')
        })
    }
})
</script>

<style scoped>
.main {
    height: 100vh;
    font-family: Arial, Helvetica, sans-serif;
}

.main-header {
    height: 60px;
}

.icon-display {
    display: flex;
    align-items: center;
    height: 60px;
    min-width: 270px;
}

.icon-img {
    height: 50px;
    width: 50px;
    margin-left: 1vh;
}

.icon-title {
    font-size: 20px;
    margin-left: 20px;
}
</style>
