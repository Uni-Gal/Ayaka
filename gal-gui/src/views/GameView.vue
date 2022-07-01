<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
</script>

<script lang="ts">
export default {
    data(): { action: Action } {
        return {
            action: { line: "", character: null, switches: [] }
        }
    },
    async created() {
        await this.next_run()
    },
    methods: {
        async next_run() {
            let res = await invoke<Action | null>("next_run")
            if (res != null) {
                this.action = res
            }
        }
    }
}

interface Action {
    line: string,
    character: string | null,
    switches: Array<Switch>,
}

interface Switch {
    text: string,
    enabled: boolean,
}

</script>

<template>
    <p>{{ action.character }}</p>
    <p>{{ action.line }}</p>
</template>
