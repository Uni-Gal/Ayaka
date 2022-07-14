<script setup lang="ts">
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
</script>

<script lang="ts">
export default {
    props: { icon: String, btnclass: String },
    emits: ["click"]
}
</script>

<template>
    <button v-bind:class='`btn btn-primary btn-command ${btnclass}`' v-on:click='$emit("click")'>
        <FontAwesomeIcon v-bind:icon='`fas fa-${icon}`'></FontAwesomeIcon>
    </button>
</template>

<style>
.btn-command {
    width: 2.5em;
    height: 2.5em;
}
</style>
