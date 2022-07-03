<script setup lang="ts">
import 'bootstrap/dist/css/bootstrap.min.css'
import { appWindow } from "@tauri-apps/api/window";
</script>

<script lang="ts">
export default {
    async created() {
        appWindow.listen("tauri://close-requested", async () => {
            await this.quit()
        })
    },
    methods: {
        async quit() {
            const confirmed = await this.$vbsModal.confirm({
                title: this.$t("quit"),
                message: this.$t("quitConfirm"),
            })
            if (confirmed) {
                await appWindow.close()
            }
        }
    }
}
</script>

<template>
    <router-view />
</template>

<style>
@import '@/assets/base.css';
</style>
