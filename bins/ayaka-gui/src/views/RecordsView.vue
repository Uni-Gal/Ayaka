<script setup lang="ts">
import { get_records, start_record, save_record_to, merge_lines, ActionText } from '../interop'
import IconButton from '../components/IconButton.vue';
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            op: this.$route.params.op,
            records: [] as ActionText[],
        }
    },
    async created() {
        this.records = await get_records()
    },
    methods: {
        async on_record_click(index: number) {
            if (this.op == "load") {
                await start_record(this.$i18n.locale, index)
                await this.$router.replace("/game")
            } else if (this.op == "save") {
                await save_record_to(index)
                await this.$router.back()
            } else {
                console.warn("Invalid op: %s", this.op)
            }
        }
    }
}
</script>

<template>
    <div class="content-below-command">
        <ul class="list-group list-group-flush">
            <li class="list-group-item list-group-item-action record-item" v-for="(rec, i) in records"
                @click="on_record_click(i)">
                <span v-html="merge_lines((rec.text))"></span>
            </li>
            <li class="list-group-item list-group-item-action record-item" @click="on_record_click(records.length)"
                :hidden='op != "save"'>
                Add new record
            </li>
        </ul>
    </div>
    <div>
        <IconButton icon="arrow-left" @click="$router.back()"></IconButton>
    </div>
</template>

<style>
.record-item {
    cursor: pointer;
}
</style>
