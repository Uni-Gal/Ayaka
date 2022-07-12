<script setup lang="ts">
import { RawContext, get_records, start_record, save_record_to } from '../interop'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            op: this.$route.params.op,
            records: [] as RawContext[],
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
                v-on:click="on_record_click(i)">
                <span v-html="rec.history[rec.history.length - 1].data.line"></span>
            </li>
            <li class="list-group-item list-group-item-action record-item" v-on:click="on_record_click(records.length)"
                v-bind:hidden='op != "save"'>
                Add new record
            </li>
        </ul>
    </div>
    <div>
        <button class="btn btn-outline-primary btn-command" v-on:click="$router.back()">
            <FontAwesomeIcon icon="fas fa-arrow-left"></FontAwesomeIcon>
        </button>
    </div>
</template>

<style>
.record-item {
    cursor: pointer;
}
</style>
