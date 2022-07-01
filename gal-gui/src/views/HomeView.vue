<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
</script>

<script lang="ts">
export default {
    data() {
        return {
            title: ""
        }
    },
    async created() {
        const res = await invoke<{ title: string }>("info")
        this.title = res.title
    }
}
async function new_game() {
    await invoke<void>("start_new")
}
</script>

<template>
    <h1 class="gal-home-title">{{ title }}</h1>
    <div class="d-grid gap-2 col-4 mx-auto">
        <router-link class="btn btn-primary" @click="new_game()" to="/game">New game</router-link>
        <router-link class="btn btn-primary" to="/about">About</router-link>
    </div>
</template>

<style>
.gal-home-title {
    text-align: center;
}
</style>
