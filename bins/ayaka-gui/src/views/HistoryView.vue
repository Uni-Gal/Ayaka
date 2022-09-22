<script setup lang="ts">
import { merge_lines, history, ActionText } from '../interop'
import ActionCard from '../components/ActionCard.vue'
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            records: [] as ActionText[]
        }
    },
    async mounted() {
        let records = await history()
        this.records = records.filter(action => action.type == "Text").map(action => action.data as ActionText)
    }
}
</script>

<template>
    <div class="content-full container-history" @click="$router.back">
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
