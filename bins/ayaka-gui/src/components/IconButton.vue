<script lang="ts">
export default {
    props: { icon: String, btnclass: String },
    emits: ["click"]
}
</script>

<template>
    <button :class='`btn btn-primary btn-command ${btnclass}`' @click='$emit("click")'>
        <font-awesome-icon :icon='`fas fa-${icon}`' />
    </button>
</template>

<style>
.btn-command {
    width: 2.5em;
    height: 2.5em;
}
</style>
