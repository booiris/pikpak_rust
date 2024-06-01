import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            name: 'login',
            component: () => import('@/views/LoginView.vue')
        },
        {
            path: '/main',
            component: () => import('@/views/MainView.vue'),
            children: [
                {
                    path: 'file_list',
                    name: 'file_list',
                    component: () => import('@/components/main/FileList.vue')
                },
                {
                    path: 'file_downloading',
                    name: 'file_downloading',
                    component: () => import('@/components/main/FileDownloading.vue')
                }
            ]
        }
    ]
})

export default router
