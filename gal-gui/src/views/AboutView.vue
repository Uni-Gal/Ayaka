<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
</script>

<script lang="ts">
export default {
    data() {
        return {
            title: "",
            author: ""
        }
    },
    async created() {
        const res = await invoke<{ title: string, author: string }>("info")
        this.title = res.title
        this.author = res.author
    }
}
</script>

<template>
    <div class="content">
        <h2>About {{ title }}</h2>
        <p>Author: {{ author }}</p>
        <h2>About gal-gui</h2>
        <p>This is a sample GUI frontend of gal project.</p>
    </div>
</template>
