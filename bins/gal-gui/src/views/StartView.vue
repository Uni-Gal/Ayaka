<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { OpenGameStatus, OpenGameStatusType, open_game, choose_locale, get_settings, set_locale } from '../interop'
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            text: "",
            progress: 0,
        }
    },
    async created() {
        listen('gal://open_status', async (e) => {
            console.log(e.payload)
            const status = e.payload as OpenGameStatus
            [this.text, this.progress] = this.status_to_text(status)
            switch (OpenGameStatusType[status.t]) {
                case OpenGameStatusType.LoadRecords:
                    await this.process_settings()
                    break
                case OpenGameStatusType.Loaded:
                    this.$router.replace("/home")
                    break
            }
        })
        await open_game()
    },
    methods: {
        status_to_text(s: OpenGameStatus): [string, number] {
            const step = 100 / 6
            console.log(s)
            const t = OpenGameStatusType[s.t]
            switch (t) {
                case OpenGameStatusType.LoadSettings:
                    return ["Loading settings...", step * (t + 1)]
                case OpenGameStatusType.LoadProfile:
                    return [`Loading profile "${s.data as unknown as string}"...`, step * (t + 1)]
                case OpenGameStatusType.CreateRuntime:
                    return ["Creating runtime...", step * (t + 1)]
                case OpenGameStatusType.LoadPlugin:
                    const data = s.data as unknown as [string, number, number];
                    const percent = data[1] / data[2];
                    return [`Loading plugin "${data[0]}"... (${data[1] + 1}/${data[2]})`, step * (t + 1) + percent * step]
                case OpenGameStatusType.LoadRecords:
                    return ["Loading records...", step * (t + 1)]
                case OpenGameStatusType.Loaded:
                    return ["Loaded.", step * (t + 1)]
                default:
                    return ["", 0]
            }
        },
        async process_settings() {
            const settings = await get_settings();
            console.log(settings)
            let loc = settings?.lang
            if (loc === undefined || loc.length == 0) {
                loc = await choose_locale(this.$i18n.availableLocales)
            }
            if (loc) {
                if (this.$i18n.availableLocales.includes(loc)) {
                    this.$i18n.locale = loc
                    await set_locale(loc)
                } else {
                    console.error("Wrong locale %s", loc)
                }
            }
        }
    }
}
</script>

<template>
    <div class="progress progress-bottom">
        <div class="progress-bar" role="progressbar" v-bind:style='`width: ${progress}%`'>{{ text }}</div>
    </div>
</template>

<style>
.progress-bottom {
    position: absolute;
    top: 100%;
    width: 100%;
    transform: translateY(-100%);
}
</style>
