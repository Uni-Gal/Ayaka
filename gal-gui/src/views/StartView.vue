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
            console.log(s)
            switch (OpenGameStatusType[s.t]) {
                case OpenGameStatusType.LoadProfile:
                    return [`Loading profile "${s.data as unknown as string}"...`, 25]
                case OpenGameStatusType.CreateRuntime:
                    return ["Creating runtime...", 50]
                case OpenGameStatusType.LoadPlugin:
                    const data = s.data as unknown as [string, number, number];
                    const percent = data[1] / data[2];
                    return [`Loading plugin "${data[0]}"... (${data[1] + 1}/${data[2]})`, 75 + percent * 25]
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
