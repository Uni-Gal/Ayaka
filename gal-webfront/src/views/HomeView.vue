<script setup lang="ts">
import axios from 'axios'
</script>

<script lang="ts">
export default {
  data() {
    return {
      title: ""
    }
  },
  created() {
    axios.get<{ title: string }>("/api/info").then(res => {
      this.title = res.data.title
    })
  }
}
</script>

<template>
  <div class="container">
    <h1 class="gal-home-title">{{ title }}</h1>
    <div class="d-grid gap-2 col-6 mx-auto">
      <router-link class="btn btn-primary" to="/about">About</router-link>
    </div>
  </div>
</template>

<style>
.gal-home-title {
  text-align: center;
}
</style>
