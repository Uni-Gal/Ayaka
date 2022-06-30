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
  <h1>About {{ title }}</h1>
  <p>Author: {{ author }}</p>
</template>
