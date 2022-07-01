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
    },
    methods: {
        async new_game() {
            await invoke<void>("start_new")
        }
    }
}
</script>

<template>
    <div class="content">
        <div class="d-grid gap-4 col-4 mx-auto">
            <h1>{{ title }}</h1>
            <router-link class="btn btn-primary" v-on:click="new_game()" to="/game">New game</router-link>
            <router-link class="btn btn-primary" to="/about">About</router-link>
        </div>
    </div>
</template>
