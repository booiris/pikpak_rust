<template>
    <div v-if="tableLoading">
        <el-skeleton :rows="12" animated :loading="tableLoading" style="margin-top: 20px" />
    </div>
    <span v-else>
        <el-empty v-if="tableData.length === 0" description="No Downloaded Task" />
        <el-table v-else :data="tableData" style="width: 100%">
            <el-table-column prop="fileName" label="Name" width="500" show-overflow-tooltip />
            <el-table-column prop="human_total" label="total" show-overflow-tooltip />
            <el-table-column prop="avg_speed" label="avg speed" show-overflow-tooltip />
            <el-table-column prop="download_time" label="duration" show-overflow-tooltip />
            <el-table-column label="">
                <template #default="scope">
                    <el-button
                        type="danger"
                        :icon="Delete"
                        @click="handleDelete(scope.row.id, scope.row.fileName)"
                    ></el-button>
                </template>
            </el-table-column>
        </el-table>
    </span>
</template>

<script setup lang="ts">
import { Filter } from '@/api'
import { mgetDownloadStatusApi } from '@/services/mget_download_status'
import type { AxiosError } from 'axios'
import { HumanizeDuration, HumanizeDurationLanguage } from 'humanize-duration-ts'
import { sortBy } from 'lodash'
import prettyBytes from 'pretty-bytes'
import { h, onMounted, onUnmounted, reactive, ref } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import { ElCheckbox, ElMessage, ElMessageBox, type CheckboxValueType } from 'element-plus'
import { downloadRemoveApi } from '@/services/download_remove'

interface tableDataType {
    fileName: string
    total: number
    human_total: string
    now: number
    current_speed: number
    download_time: string
    status: string
    id: string
    avg_speed: string
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

async function get_downloaded_status(): Promise<tableDataType[]> {
    tableLoading.value = false

    let res: tableDataType[] = []

    await mgetDownloadStatusApi([Filter.Completed])
        .then((data) => {
            data.data.download_status.forEach((item) => {
                res.push({
                    fileName: item.remote_file_name,
                    total: item.total,
                    now: item.downloaded,
                    current_speed: item.current_speed,
                    download_time: humanizer.humanize(item.downloaded_time * 1000, {
                        language: 'shortEn'
                    }),
                    human_total: prettyBytes(item.total),
                    status: item.status.toString(),
                    id: item.file_id,
                    avg_speed: prettyBytes(item.total / item.downloaded_time) + '/s'
                })
            })
        })
        .catch((err: AxiosError) => {
            console.error(err)
        })

    res = sortBy(res, ['fileName'])
    return res
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
    let tempData = await get_downloaded_status()

    tableData.splice(0, tableData.length, ...tempData)

    query = setInterval(async () => {
        let tempData = await get_downloaded_status()
        tableData.splice(0, tableData.length, ...tempData)
    }, 1000)
})

onUnmounted(() => {
    if (query) {
        clearInterval(query)
    }
})
</script>
