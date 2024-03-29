<script setup lang="ts">
import { listen, Event as TauriEvent, UnlistenFn } from '@tauri-apps/api/event';
import { OpenGameStatus, OpenGameStatusType, open_game, choose_locale, get_settings, set_locale } from '../interop'
import { Modal } from 'bootstrap'
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            text: "",
            error: "",
            progress: 0,
            unlisten_fn: undefined as UnlistenFn | undefined,
            modal: undefined as Modal | undefined
        }
    },
    async mounted() {
        this.modal = new Modal(this.$refs.errorModal as HTMLElement)
        this.unlisten_fn = await listen('ayaka://open_status', this.on_open_status)
        await this.open_game()
    },
    unmounted() {
        if (this.unlisten_fn) {
            this.unlisten_fn()
            this.unlisten_fn = undefined
        }
    },
    methods: {
        async open_game() {
            try {
                await open_game()
            }
            catch (e) {
                if (e instanceof Error) {
                    this.error = e.message
                } else {
                    this.error = JSON.stringify(e)
                }
                this.modal?.show()
            }
        },
        async reopen_game() {
            this.modal?.hide()
            await this.open_game()
        },
        async on_open_status(e: TauriEvent<OpenGameStatus>) {
            console.log(e.payload)
            const status = e.payload;
            [this.text, this.progress] = this.status_to_text(status);
            let anime = (this.$refs.logo as HTMLElement).animate([
                { rotate: `${this.rotate_degree()}deg` }
            ], {
                duration: 500,
                fill: "forwards",
                easing: "ease-out"
            })
            anime.onfinish = async () => {
                switch (OpenGameStatusType[status.t]) {
                    case OpenGameStatusType.LoadRecords:
                        await this.process_settings()
                        break
                    case OpenGameStatusType.Loaded:
                        this.$router.replace("/home")
                        break
                }
            }
        },
        status_to_text(s: OpenGameStatus): [string, number] {
            const step = 100 / 10
            const t = OpenGameStatusType[s.t]
            switch (t) {
                case OpenGameStatusType.LoadProfile:
                    return [`Loading profile...`, step * (t + 1)]
                case OpenGameStatusType.CreateRuntime:
                    return ["Creating runtime...", step * (t + 1)]
                case OpenGameStatusType.LoadPlugin:
                    const data = s.data as unknown as [string, number, number]
                    return [`Loading plugin ${data[0]}...`, step * (t + 1) + data[1] / data[2] * step]
                case OpenGameStatusType.GamePlugin:
                    return ["Preprocessing game...", step * (t + 1)]
                case OpenGameStatusType.LoadResource:
                    return ["Loading resources...", step * (t + 1)]
                case OpenGameStatusType.LoadParagraph:
                    return ["Loading paragraphs...", step * (t + 1)]
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
            const settings = await get_settings()
            console.log(settings)
            let loc: string | undefined = settings.lang
            if (!loc || !this.$i18n.availableLocales.includes(loc)) {
                loc = await choose_locale(this.$i18n.availableLocales)
            }
            if (loc) {
                console.log("Set locale to %s", loc)
                this.$i18n.locale = loc
                await set_locale(loc)
            }
        },
        rotate_degree() {
            return this.progress / 100 * 360
        }
    }
}
</script>

<template>
    <div class="content-logo">
        <img ref="logo" width="300" height="300" src="../assets/logo.png" alt="Logo" />
        <p class="fw-bolder" style="font-size: 300%">Just Ayaka.</p>
    </div>
    <div class="progress progress-bottom">
        <div class="progress-bar" role="progressbar" :style='`width: ${progress}%`'>{{ text }}</div>
    </div>

    <div class="modal fade" ref="errorModal" tabindex="-1">
        <div class="modal-dialog">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">{{ $t("error") }}</h5>
                </div>
                <div class="modal-body">{{ error }}</div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-primary" @click="reopen_game">
                        {{ $t("dialogOk") }}
                    </button>
                </div>
            </div>
        </div>
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
    translate: -50% -50%;
    text-align: center;
}
</style>
