<script setup lang="ts">
import 'bootstrap-dark-5/dist/css/bootstrap-dark.min.css'
import { appWindow } from "@tauri-apps/api/window"
import { choose_locale, get_locale, save_locale } from './interop';
</script>

<script lang="ts">
export default {
    async created() {
        const loc = get_locale() ?? await choose_locale(this.$i18n.availableLocales)
        if (loc != null) {
            if (this.$i18n.availableLocales.includes(loc)) {
                this.$i18n.locale = loc
                save_locale(loc)
            } else {
                console.error("Wrong locale %s", loc)
            }
        }
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
