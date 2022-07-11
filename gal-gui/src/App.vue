<script setup lang="ts">
import 'bootstrap-dark-5/dist/css/bootstrap-dark.min.css'
import { appWindow } from "@tauri-apps/api/window"
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
                leftBtnText: this.$t("dialogNo"),
                rightBtnText: this.$t("dialogYes"),
            })
            if (confirmed) {
                await appWindow.close()
            }
        }
    }
}
</script>

<template>
    <router-view @quit="quit" />
</template>

<style>
@import './assets/base.css';
</style>
