<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
import { appWindow } from "@tauri-apps/api/window";
import router from '../router';
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
            if (await invoke<boolean>("next_run")) {
                router.push("/game")
            }
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
            <button class="btn btn-primary" v-on:click="new_game">New game</button>
            <router-link class="btn btn-primary" to="/about">About</router-link>
            <button class="btn btn-primary" v-on:click="quit">Quit</button>
        </div>
    </div>
</template>
