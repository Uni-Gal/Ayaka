<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { OpenGameStatus, OpenGameStatusType } from '../interop'
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
    created() {
        listen('gal://open_status', e => {
            console.log("open_status")
            console.log(e.payload)
            const status = e.payload as OpenGameStatus
            this.text = this.status_to_text(status)
            if (status.t == "Loaded") {
                router.replace("/home")
            }
        })
    },
    methods: {
        status_to_text(s: OpenGameStatus): string {
            switch (OpenGameStatusType[s.t]) {
                case OpenGameStatusType.LoadProfile:
                    return "Loading profile..."
                case OpenGameStatusType.CreateRuntime:
                    return "Creating runtime..."
                case OpenGameStatusType.LoadPlugin:
                    return "Loading plugin..."
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
            <h1>{{ text }}</h1>
        </div>
    </div>
</template>
