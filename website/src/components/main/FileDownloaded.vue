<template>
    <div v-if="tableLoading">
        <el-skeleton :rows="12" animated :loading="tableLoading" style="margin-top: 20px" />
    </div>
    <span v-else>
        <el-empty v-if="tableData.length === 0" description="No Downloaded Task" />
        <el-table v-else :data="tableData" style="width: 100%">
            <el-table-column prop="fileName" label="Name" width="200" show-overflow-tooltip />
            <el-table-column prop="size" label="Size" width="180" show-overflow-tooltip />
            <el-table-column
                prop="download_last_time"
                label="costTime"
                width="180"
                show-overflow-tooltip
            />
            <el-table-column
                prop="download_end_timestamp"
                label="downloadTimeStamp"
                show-overflow-tooltip
            />
        </el-table>
    </span>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

interface tableDataType {
    fileName: string
    size: string
    download_last_time: string
    download_end_timestamp: string
}
const tableData = reactive<tableDataType[]>([])
const tableLoading = ref(true)

async function get_downloaded_status(): Promise<tableDataType[]> {
    tableLoading.value = false
    return [
        {
            fileName: 'test',
            size: '100',
            download_last_time: '10',
            download_end_timestamp: '11111111'
        }
    ]
}

onMounted(async () => {
    let tempData = await get_downloaded_status()
    tableData.splice(0, tableData.length, ...tempData)
})
</script>
