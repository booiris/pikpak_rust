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
                    <div v-if="tableLoading">
                        <el-skeleton :rows="8" animated :loading="tableLoading" />
                    </div>
                    <el-table v-else :data="tableData" style="width: 100%" :cell-style="cellStyle">
                        <el-table-column prop="fileName" label="Name" width="500" style="color: red;" />
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
import { onMounted, reactive, ref, type CSSProperties } from 'vue'
import { parseISO, format } from 'date-fns';
import prettyBytes from 'pretty-bytes';

interface tableDataType {
    fileName: string
    size: string
    updateTime: string
    kind: string
}

const tableData = reactive<tableDataType[]>([])
const tableLoading = ref(true)

async function get_file_status(path: string): Promise<tableDataType[]> {
    const data = await remoteListApi(path).catch((err: AxiosError) => {
        console.error(err)
        let error = parseApiError(err)
        ElMessage.error({
            showClose: true,
            message: 'get remote file list failed, err: ' + error,
            duration: 5000
        })
    })
    tableLoading.value = false
    if (!data) {
        return []
    }
    console.log(data)
    let tempData = []
    for (let file_status of data.data.files_info) {
        const date = parseISO(file_status.modified_time);
        let size = "-"
        if (file_status.kind != "drive#folder") {
            size = prettyBytes(Number(file_status.size))
        }
        tempData.push({
            fileName: file_status.name,
            size: size,
            updateTime: format(date, 'yyyy-MM-dd HH:mm'),
            kind: file_status.kind,
        })
    }
    return tempData
}

function cellStyle({ row, columnIndex }: { row: tableDataType, column: any, rowIndex: number, columnIndex: number }): CSSProperties {
    if (row.kind == "drive#folder" && columnIndex == 0) {
        return {
            color: "#409EFF"
        }
    } else {
        return {}
    }
}

onMounted(async () => {
    let tempData = await get_file_status('/')
    tableData.splice(0, tableData.length, ...tempData)
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
