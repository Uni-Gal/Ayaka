<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { OpenGameStatus, OpenGameStatusType, open_game } from '../interop'
import router from '../router';
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
        listen('gal://open_status', e => {
            console.log(e.payload)
            const status = e.payload as OpenGameStatus
            [this.text, this.progress] = this.status_to_text(status)
            if (OpenGameStatusType[status.t] == OpenGameStatusType.Loaded) {
                router.replace("/home")
            }
        })
        await open_game()
    },
    methods: {
        status_to_text(s: OpenGameStatus): [string, number] {
            switch (OpenGameStatusType[s.t]) {
                case OpenGameStatusType.LoadProfile:
                    return [`Loading profile "${s.text}"...`, 25]
                case OpenGameStatusType.CreateRuntime:
                    return ["Creating runtime...", 50]
                case OpenGameStatusType.LoadPlugin:
                    return [`Loading plugin "${s.text}"...`, 75]
                case OpenGameStatusType.Loaded:
                    return ["Loaded.", 100]
                default:
                    return ["", 0]
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
