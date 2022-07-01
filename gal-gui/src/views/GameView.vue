<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
import { setTimeout } from 'timers-promises'
</script>

<script lang="ts">
function action_default(): Action {
    return { line: "", character: null, switches: [] }
}

export default {
    data(): { action: Action, action_data: Action } {
        return {
            action: action_default(),
            action_data: action_default(),
        }
    },
    async created() {
        await this.next_run()
    },
    methods: {
        async next_run() {
            let res = await invoke<Action | null>("next_run")
            if (res != null) {
                this.action_data = res
                await this.type_text()
            } else {
                location.href = "/"
            }
        },
        async switch_run(i: number) {
            await invoke<void>("switch", { i: i })
            await this.next_run()
        },
        async type_text() {
            this.action.line = ""
            this.action.switches = []
            this.action.character = this.action_data.character
            while (this.action.line.length < this.action_data.line.length) {
                this.action.line += this.action_data.line[this.action.line.length]
                await setTimeout(10)
            }
        },
        async onclick() {
            if (this.action.line.length < this.action_data.line.length) {
                this.action.line = this.action_data.line
            } else if (this.action.switches.length < this.action_data.switches.length) {
                this.action.switches = this.action_data.switches
            } else {
                await this.next_run()
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
    <div v-on:click="onclick()">
        <div class="card bottom">
            <div class="card-header char">
                <h4 class="card-title">{{ action.character }}</h4>
            </div>
            <div class="card-body lines">
                <p class="h4 card-text">{{ action.line }}</p>
            </div>
        </div>
    </div>
    <div class="container-switches" v-bind:hidden="action.switches.length == 0">
        <div class="switches">
            <div class="switches-center">
                <div class="d-grid gap-4 col-8 mx-auto">
                    <button class="btn btn-primary" v-for="s in action.switches"
                        v-on:click="switch_run(action.switches.indexOf(s))" v-bind:disabled="!s.enabled">
                        {{ s.text }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style>
.bottom {
    position: absolute;
    bottom: 0;
    width: 100%;
}

.char {
    height: 3em;
}

.lines {
    height: 8em;
}

.container-switches {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    width: 100%;
    background-color: #00000077;
}

.switches {
    position: absolute;
    width: 100%;
    height: calc(100% - 11em);
}

.switches-center {
    position: absolute;
    width: 100%;
    top: 50%;
    transform: translateY(-50%);
}
</style>
