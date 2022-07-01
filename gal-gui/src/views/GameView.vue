<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
</script>

<script lang="ts">
export default {
    data() { },
    async created() {
        const query = location.search
        const params = new URLSearchParams(query)
        const new_game = Boolean(params.get("new"))
        if (new_game) {
            await invoke<void>("start_new")
        }
        location.search = ""
    }
}
</script>

<template>
    <h1>New game</h1>
</template>
