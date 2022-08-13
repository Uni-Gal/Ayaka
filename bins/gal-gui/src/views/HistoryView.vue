<script setup lang="ts">
import { merge_lines, history, Action } from '../interop'
import ActionCard from '../components/ActionCard.vue'
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            records: [] as Action[]
        }
    },
    async mounted() {
        this.records = await history()
    }
}
</script>

<template>
    <div class="content-full container-history" v-on:click="$router.back">
        <ul class="list-group">
            <li class="list-group-item" v-for="h in records">
                <ActionCard :ch="h.character" :line="merge_lines(h.line)"></ActionCard>
            </li>
        </ul>
    </div>
</template>

<style>
.container-history {
    overflow-y: scroll;
}
</style>
