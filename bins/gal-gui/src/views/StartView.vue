<script setup lang="ts">
import { listen, Event as TauriEvent, UnlistenFn } from '@tauri-apps/api/event';
import { OpenGameStatus, OpenGameStatusType, open_game, choose_locale, get_settings, set_locale } from '../interop'
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            text: "",
            progress: 0,
            unlisten_fn: null as UnlistenFn | null
        }
    },
    async mounted() {
        this.unlisten_fn = await listen('gal://open_status', this.on_open_status)
        await open_game()
    },
    unmounted() {
        if (this.unlisten_fn) {
            this.unlisten_fn()
            this.unlisten_fn = null
        }
    },
    methods: {
        async on_open_status(e: TauriEvent<OpenGameStatus>) {
            console.log(e.payload)
            const status = e.payload;
            [this.text, this.progress] = this.status_to_text(status)
            switch (OpenGameStatusType[status.t]) {
                case OpenGameStatusType.LoadRecords:
                    await this.process_settings()
                    break
                case OpenGameStatusType.Loaded:
                    this.$router.replace("/home")
                    break
            }
        },
        status_to_text(s: OpenGameStatus): [string, number] {
            const step = 100 / 7
            const t = OpenGameStatusType[s.t]
            switch (t) {
                case OpenGameStatusType.LoadProfile:
                    return [`Loading profile "${s.data as unknown as string}"...`, step * (t + 1)]
                case OpenGameStatusType.CreateRuntime:
                    return ["Creating runtime...", step * (t + 1)]
                case OpenGameStatusType.LoadPlugin:
                    const data = s.data as unknown as [string, number, number];
                    const percent = data[1] / data[2];
                    return [`Loading plugin "${data[0]}"... (${data[1] + 1}/${data[2]})`, step * (t + 1) + percent * step]
                case OpenGameStatusType.LoadSettings:
                    return ["Loading settings...", step * (t + 1)]
                case OpenGameStatusType.LoadGlobalRecords:
                    return ["Loading global records...", step * (t + 1)]
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
    <img class="content-logo" src="../assets/logo.png" alt="Logo" />
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

.content-logo {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: fit-content;
    height: fit-content;
    text-align: center;
}
</style>
