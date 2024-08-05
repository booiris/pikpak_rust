<template>
    <el-page-header @back="handleBack">
        <template #breadcrumb>
            <el-breadcrumb separator="/">
                <el-breadcrumb-item v-for="(item, index) in pathList" :key="index">
                    <el-tooltip :content="item" placement="top">
                        <el-text
                            truncated
                            style="max-width: 5rem; cursor: pointer"
                            @click="jumpPath"
                            :data-key="index"
                        >
                            {{ item }}
                        </el-text>
                    </el-tooltip>
                </el-breadcrumb-item>
            </el-breadcrumb>
        </template>
        <template #content>
            <span class="text-large font-400 mr-3" style="color: #409eff">
                {{ pathList.at(-1) }}
            </span>
        </template>
    </el-page-header>
    <div v-if="tableLoading">
        <el-skeleton :rows="12" animated :loading="tableLoading" style="margin-top: 20px" />
    </div>
    <el-table
        v-else
        :data="tableData"
        style="width: 100%"
        :cell-style="cellStyle"
        @row-click="handleRawClick"
    >
        <el-table-column prop="fileName" label="Name" width="500" show-overflow-tooltip />
        <el-table-column prop="size" label="Size" width="180" show-overflow-tooltip />
        <el-table-column prop="updateTime" label="Update Time" show-overflow-tooltip />
    </el-table>
</template>

<script setup lang="ts">
import { remoteListApi } from '@/services/remote_list'
import { parseApiError } from '@/utils/handle_error'
import type { AxiosError } from 'axios'
import { ElMessage } from 'element-plus'
import { onMounted, reactive, ref, watch, type CSSProperties } from 'vue'
import { parseISO, format } from 'date-fns'
import prettyBytes from 'pretty-bytes'
import _ from 'lodash'
import { ElMessageBox } from 'element-plus'
import { downloadBeginApi } from '@/services/download_begin'

interface tableDataType {
    fileName: string
    size: string
    updateTime: string
    kind: string
    id: string
}

const tableData = reactive<tableDataType[]>([])
const tableLoading = ref(true)

async function get_file_status(path: string): Promise<tableDataType[]> {
    tableLoading.value = true
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
    console.debug(data)
    let tempData = []
    for (let file_status of data.data.files_info) {
        const date = parseISO(file_status.modified_time)
        let size = '-'
        if (file_status.kind != 'drive#folder') {
            size = prettyBytes(Number(file_status.size))
        }
        tempData.push({
            fileName: file_status.name,
            size: size,
            updateTime: format(date, 'yyyy-MM-dd HH:mm'),
            kind: file_status.kind,
            id: file_status.id
        })
    }
    return tempData
}

function cellStyle({
    row,
    columnIndex
}: {
    row: tableDataType
    column: any
    rowIndex: number
    columnIndex: number
}): CSSProperties {
    if (row.kind == 'drive#folder' && columnIndex == 0) {
        return {
            color: '#409EFF',
            cursor: 'pointer'
        }
    } else {
        return {
            cursor: 'pointer'
        }
    }
}

onMounted(async () => {
    let tempData = await get_file_status('/')
    tableData.splice(0, tableData.length, ...tempData)
})

function handleRawClick(row: tableDataType) {
    if (row.kind == 'drive#folder') {
        pathList.value.push(row.fileName)
    }
    if (row.kind == 'drive#file') {
        ElMessageBox.confirm('Download "' + row.fileName + '" , Continue?', 'Download Confirm', {
            confirmButtonText: 'OK',
            cancelButtonText: 'Cancel',
            type: 'info'
        })
            .then(async () => {
                await downloadBeginApi(row.id, './', 'test.mp4')
                    .then((data) => {
                        if (data.data.code == 1) {
                            ElMessage({
                                type: 'info',
                                message: '"' + row.fileName + '" Already Download'
                            })
                            return
                        } else {
                            ElMessage({
                                type: 'success',
                                message: '"' + row.fileName + '" Download Start'
                            })
                        }
                    })
                    .catch((err: AxiosError) => {
                        console.error(err)
                        let error = parseApiError(err)
                        ElMessage.error({
                            showClose: true,
                            message: 'download file <' + row.fileName + '> failed, err: ' + error,
                            duration: 5000
                        })
                    })
            })
            .catch(() => {
                ElMessage({
                    type: 'info',
                    message: '"' + row.fileName + '" Download Canceled'
                })
            })
    }
}

// -------------------- File breadcrumb --------------------

const pathList = ref<string[]>(['Root'])
let oldPath: string[] = []

function jumpPath(event: MouseEvent) {
    const target = event.target as HTMLElement
    const key = target.getAttribute('data-key')
    pathList.value = pathList.value.slice(0, Number(key) + 1)
}

function handleBack() {
    if (pathList.value.length == 1) {
        return
    }
    pathList.value.pop()
}

watch(
    pathList,
    async (newPath) => {
        if (_.isEqual(newPath, oldPath)) {
            return
        }
        let tempData = await get_file_status('/' + newPath.slice(1).join('/'))
        tableData.splice(0, tableData.length, ...tempData)
        oldPath = _.cloneDeep(newPath)
    },
    { deep: true }
)

// -------------------- File breadcrumb --------------------
</script>

<style scoped>
.el-table {
    height: 80vh;
}
</style>
