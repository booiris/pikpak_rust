<template>
    <div v-if="tableLoading">
        <el-skeleton :rows="12" animated :loading="tableLoading" style="margin-top: 20px" />
    </div>
    <span v-else>
        <el-empty v-if="tableData.length === 0" description="No Downloading Task" />
        <el-table v-else :data="tableData" style="width: 100%">
            <el-table-column prop="fileName" label="Name" width="200" show-overflow-tooltip />
            <el-table-column label="progress" width="350">
                <template #default="scope">
                    <el-progress
                        :percentage="
                            Math.min(parseFloat(scope.row.now) / parseFloat(scope.row.total), 1.0) *
                            100
                        "
                        :stroke-width="15"
                        striped-flow
                        striped
                        :duration="1.5"
                        :format="formatProgress"
                    />
                </template>
            </el-table-column>
            <el-table-column label="speed" prop="current_speed" show-overflow-tooltip />
            <el-table-column label="downloaded" prop="downloaded" show-overflow-tooltip />
            <el-table-column label="size" prop="human_total" show-overflow-tooltip />
            <el-table-column label="download time" prop="download_time" show-overflow-tooltip />
            <el-table-column label="action" width="100">
                <template #default="scope">
                    <el-button
                        v-if="isError(scope.row.status) || scope.row.status == 'Paused'"
                        type="primary"
                        :icon="VideoPlay"
                        circle
                        @click="handleResume(scope.row.id)"
                    />
                    <el-button
                        v-else
                        type="primary"
                        :icon="VideoPause"
                        circle
                        @click="handlePause(scope.row.id)"
                    />
                    <el-button
                        type="danger"
                        :icon="Delete"
                        circle
                        @click="handleDelete(scope.row.id, scope.row.fileName)"
                    ></el-button>
                </template>
            </el-table-column>
        </el-table>
    </span>
</template>

<script setup lang="ts">
import { Filter, type DownloadStatusEnum } from '@/api'
import { mgetDownloadStatusApi } from '@/services/mget_download_status'
import type { AxiosError } from 'axios'
import { onMounted, onUnmounted, reactive, ref } from 'vue'
import { HumanizeDurationLanguage, HumanizeDuration } from 'humanize-duration-ts'
import prettyBytes from 'pretty-bytes'
import { sortBy } from 'lodash'
import { pauseDownloadApi } from '@/services/pause_download'
import { VideoPause, VideoPlay, Delete } from '@element-plus/icons-vue'
import { ElCheckbox, ElMessage, ElMessageBox, type CheckboxValueType } from 'element-plus'
import { downloadResumeApi } from '@/services/download_resume'
import { h } from 'vue'
import { downloadRemoveApi } from '@/services/download_remove'

interface tableDataType {
    fileName: string
    total: number
    human_total: string
    now: number
    current_speed: string
    download_time: string
    status: DownloadStatusEnum
    id: string
    download_path: string
    downloaded: string
}
const tableData = reactive<tableDataType[]>([])
const tableLoading = ref(true)
const langService: HumanizeDurationLanguage = new HumanizeDurationLanguage()
langService.addLanguage('shortEn', {
    y: () => 'y',
    mo: () => 'mo',
    w: () => 'w',
    d: () => 'd',
    h: () => 'h',
    m: () => 'm',
    s: () => 's',
    ms: () => 'ms',
    decimal: ''
})
const humanizer: HumanizeDuration = new HumanizeDuration(langService)

function isError(value: DownloadStatusEnum): boolean {
    return typeof value === 'object' && value !== null && 'HasError' in value
}

async function get_downloading_status(): Promise<tableDataType[]> {
    tableLoading.value = false

    let res: tableDataType[] = []

    await mgetDownloadStatusApi([
        Filter.Downloading,
        Filter.Waiting,
        Filter.Paused,
        Filter.HasError
    ])
        .then((data) => {
            data.data.download_status.forEach((item) => {
                res.push({
                    fileName: item.remote_file_name,
                    total: item.total,
                    now: item.downloaded,
                    current_speed: prettyBytes((item.current_speed / 1000) * 8) + '/s',
                    download_time: humanizer.humanize(item.downloaded_time * 1000, {
                        language: 'shortEn'
                    }),
                    human_total: prettyBytes(item.total),
                    status: item.status,
                    id: item.file_id,
                    download_path: item.download_to_local_path,
                    downloaded: prettyBytes(item.downloaded)
                })
            })
        })
        .catch((err: AxiosError) => {
            console.error(err)
        })
    res = sortBy(res, ['fileName'])
    return res
}

async function handlePause(id: string) {
    await pauseDownloadApi(id)
        .then(() => {})
        .catch((err: AxiosError) => {
            console.error(err)
        })
}

async function handleResume(id: string) {
    await downloadResumeApi(id)
        .then(() => {})
        .catch((err: AxiosError) => {
            console.error(err)
        })
}

function formatProgress(percentage: number): string {
    return percentage.toFixed(1) + '%'
}

async function handleDelete(id: string, name: string) {
    const is_delete_file = ref<boolean>(false)

    ElMessageBox.confirm('Warning', {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        type: 'warning',
        title: 'Are you sure to delete this task?',
        message: () =>
            h(ElCheckbox, {
                label: 'Delete origin file',
                modelValue: is_delete_file.value,
                value: is_delete_file.value,
                'onUpdate:modelValue': (val: CheckboxValueType) => {
                    is_delete_file.value = val as boolean
                }
            }),
        beforeClose: async (action, instance, done) => {
            if (action === 'confirm') {
                instance.confirmButtonLoading = true
                instance.confirmButtonText = 'Loading...'
                await downloadRemoveApi(id, is_delete_file.value)
                    .then(() => {
                        ElMessage({
                            type: 'success',
                            message: 'Delete ' + name + ' Success'
                        })
                    })
                    .catch((err: AxiosError) => {
                        console.error(err)
                        ElMessage({
                            type: 'error',
                            message: 'Delete ' + name + ' Failed'
                        })
                    })
                instance.confirmButtonLoading = false
            }
            done()
        }
    })
}
let query: number | null = null
onMounted(async () => {
    let tempData = await get_downloading_status()
    tableData.splice(0, tableData.length, ...tempData)

    query = setInterval(async () => {
        let tempData = await get_downloading_status()
        tableData.splice(0, tableData.length, ...tempData)
    }, 1000)
})

onUnmounted(() => {
    if (query) {
        clearInterval(query)
    }
})
</script>
