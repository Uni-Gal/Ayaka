<script setup lang="ts">
import axios from 'axios'
</script>

<script lang="ts">
export default {
  data() {
    return {
      title: "",
      author: ""
    }
  },
  created() {
    axios.get<{ title: string, author: string }>("/api/info").then(res => {
      this.title = res.data.title
      this.author = res.data.author
    })
  }
}
</script>

<template>
  <div class="container">
    <h1>About {{ title }}</h1>
    <p>Author: {{ author }}</p>
  </div>
</template>
