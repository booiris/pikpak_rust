<template>
    <div class="main-header">
        <el-row :gutter="20">
            <el-col :span="6">
                <div class="icon-display">
                    <el-image :src="icon" class="icon-img" />
                    <el-text class="icon-title">PikPak Rust</el-text>
                </div>
            </el-col>
            <el-col :span="18">
                <!-- <div class="file-list" style="background-color: grey; height: 100%" /> -->
            </el-col>
        </el-row>
    </div>
</template>

<script setup lang="ts">
import icon from '@/assets/rust_pikpak.png'
</script>

<style scoped>
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
