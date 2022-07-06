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
        }
    },
    async created() {
        listen('gal://open_status', e => {
            console.log(e.payload)
            const status = e.payload as OpenGameStatus
            this.text = this.status_to_text(status)
            if (OpenGameStatusType[status.t] == OpenGameStatusType.Loaded) {
                router.replace("/home")
            }
        })
        await open_game()
    },
    methods: {
        status_to_text(s: OpenGameStatus): string {
            switch (OpenGameStatusType[s.t]) {
                case OpenGameStatusType.LoadProfile:
                    return `Loading profile "${s.text}"...`
                case OpenGameStatusType.CreateRuntime:
                    return "Creating runtime..."
                case OpenGameStatusType.LoadPlugin:
                    return `Loading plugin "${s.text}"...`
                case OpenGameStatusType.Loaded:
                    return "Loaded."
                default:
                    return ""
            }
        }
    }
}
</script>

<template>
    <div class="content">
        <div class="d-grid gap-4 col-4 mx-auto">
            <p>{{ text }}</p>
        </div>
    </div>
</template>
