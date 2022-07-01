<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
import { appWindow } from "@tauri-apps/api/window";
</script>

<script lang="ts">
export default {
    data() {
        return {
            title: ""
        }
    },
    async created() {
        appWindow.listen("tauri://close-requested", async () => { await this.quit() })
        const res = await invoke<{ title: string }>("info")
        this.title = res.title
    },
    methods: {
        async new_game() {
            await invoke<void>("start_new")
        },
        async quit() {
            const confirmed = await this.$vbsModal.confirm({
                title: "Quit",
                message: "Quit the game?",
            })
            if (confirmed) {
                await appWindow.close()
            }
        }
    }
}
</script>

<template>
    <div class="content">
        <div class="d-grid gap-4 col-4 mx-auto">
            <h1>{{ title }}</h1>
            <router-link class="btn btn-primary" v-on:click="new_game" to="/game">New game</router-link>
            <router-link class="btn btn-primary" to="/about">About</router-link>
            <button class="btn btn-primary" v-on:click="quit">Quit</button>
        </div>
    </div>
</template>
