<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
</script>

<script lang="ts">
export default {
    data(): { action: Action } {
        return {
            action: { line: "", character: null, switches: [] }
        }
    },
    async created() {
        await this.next_run()
    },
    methods: {
        async next_run() {
            let res = await invoke<Action | null>("next_run")
            if (res != null) {
                this.action = res
            }
        }
    }
}

interface Action {
    line: string,
    character: string | null,
    switches: Array<Switch>,
}

interface Switch {
    text: string,
    enabled: boolean,
}
</script>

<template>
    <div v-on:click="next_run()">
        <div class="card bottom">
            <div class="card-header char">
                <h4 class="card-title">{{ action.character }}</h4>
            </div>
            <div class="card-body lines">
                <p class="h4 card-text">{{ action.line }}</p>
            </div>
        </div>
    </div>
</template>

<style>
div.bottom {
    position: absolute;
    bottom: 0;
    width: 100%;
    opacity: 0.8;
}

div.char {
    height: 3em;
}

div.lines {
    height: 8em;
}
</style>
