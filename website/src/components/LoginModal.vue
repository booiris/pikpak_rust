<template>
    <div class="modal-overlay" v-if="show">
        <div class="modal">
            <div class="modal-header">
                <slot name="header"></slot>
                <button class="close-button" @click="$emit('close')">
                    <i class="fas fa-times"></i>
                </button>
            </div>
            <div class="modal-body">
                <slot name="body"></slot>
            </div>
            <div class="modal-footer">
                <slot name="footer"></slot>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue';
// import '@fortawesome/fontawesome-free/css/all.css';

const props = defineProps({
    show: {
        type: Boolean,
        required: true
    }
});

const emits = defineEmits(['close']);
</script>

<style scoped>
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
}

.modal {
    background: #fff;
    padding: 20px;
    border-radius: 10px;
    max-width: 500px;
    width: 100%;
    text-align: center;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 1.5em;
    border-bottom: 1px solid #e0e0e0;
    padding-bottom: 10px;
    margin-bottom: 20px;
}

.close-button {
    background: none;
    border: none;
    font-size: 1.5em;
    cursor: pointer;
    color: #999;
}

.close-button:hover {
    color: #666;
}

.modal-body {
    font-size: 1em;
    color: #333;
}

.modal-footer {
    margin-top: 20px;
}

.modal-footer button {
    padding: 10px 20px;
    background-color: #409eff;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    cursor: pointer;
    transition: background-color 0.3s ease, transform 0.3s ease;
}

.modal-footer button:hover {
    background-color: #66b1ff;
    transform: translateY(-2px);
}
</style>
