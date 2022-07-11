<script setup lang="ts">
import { RawContext, get_records, start_record, next_run } from '../interop'
import router from '../router';
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            records: [] as RawContext[],
        }
    },
    async created() {
        this.records = await get_records()
    },
    methods: {
        async on_record_click(index: number) {
            await start_record(this.$i18n.locale, index)
            if (await next_run()) {
                router.replace("/game")
            }
        }
    }
}
</script>

<template>
    <div class="content-full">
        <ul class="list-group">
            <li class="list-group-item" v-for="(rec, i) in records" v-on:click="on_record_click(i)">
                {{ rec.history[rec.history.length - 1].line }}
            </li>
        </ul>
    </div>
</template>
