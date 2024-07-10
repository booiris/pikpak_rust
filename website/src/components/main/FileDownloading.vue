<template>
    <div v-if="tableLoading">
        <el-skeleton :rows="12" animated :loading="tableLoading" style="margin-top: 20px" />
    </div>
    <span v-else>
        <el-empty v-if="tableData.length === 0" description="No Downloading Task" />
        <el-table v-else :data="tableData" style="width: 100%">
            <el-table-column prop="fileName" label="Name" width="200" show-overflow-tooltip />
            <el-table-column label="progress" width="600">
                <template #default="scope">
                    <el-progress
                        :percentage="
                            (parseFloat(scope.row.now) / parseFloat(scope.row.total)) * 100
                        "
                        :stroke-width="15"
                        striped-flow
                        striped
                        :duration="15"
                        text-inside
                    />
                    <div style="display: flex; align-items: center"></div>
                </template>
            </el-table-column>
        </el-table>
    </span>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

interface tableDataType {
    fileName: string
    total: string
    now: string
    current_speed: string
    download_time: string
}
const tableData = reactive<tableDataType[]>([])
const tableLoading = ref(true)

async function get_downloading_status(): Promise<tableDataType[]> {
    tableLoading.value = false
    return [
        {
            fileName: 'test',
            total: '100',
            now: '50',
            current_speed: '10',
            download_time: '10'
        }
    ]
}

onMounted(async () => {
    let tempData = await get_downloading_status()
    tableData.splice(0, tableData.length, ...tempData)
})
</script>
