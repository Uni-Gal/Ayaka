<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
import { setTimeout } from 'timers-promises'
import { Mutex, tryAcquire } from 'async-mutex'
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome"
</script>

<script lang="ts">
function action_default(): Action {
    return { line: "", character: null, switches: [] }
}

export default {
    data() {
        return {
            action: action_default(),
            action_data: action_default(),
            mutex: new Mutex(),
        }
    },
    async created() {
        document.addEventListener('keydown', this.onkeydown)
        await this.mutex.runExclusive(this.fetch_current_run)
        await this.start_type_anime()
    },
    methods: {
        go_home() {
            this.action_data = action_default()
            this.action = action_default()
            location.replace("/")
        },
        // Should be called in mutex
        async fetch_current_run() {
            let res = await invoke<Action | null>("current_run")
            if (res != null) {
                this.action_data = res
            } else {
                this.go_home()
            }
        },
        // Should be called in mutex
        async fetch_next_run() {
            await invoke<boolean>("next_run")
            await this.fetch_current_run()
        },
        async switch_run(i: number) {
            await invoke<void>("switch", { i: i })
            await this.mutex.runExclusive(this.fetch_next_run)
            await this.start_type_anime()
        },
        // Shouldn't be called in mutex
        async start_type_anime() {
            this.action.line = ""
            this.action.switches = []
            this.action.character = this.action_data.character
            while (this.action.line.length < this.action_data.line.length) {
                this.action.line += this.action_data.line[this.action.line.length]
                await setTimeout(10)
            }
        },
        async next() {
            if (this.action.switches.length == 0) {
                const new_text = await tryAcquire(this.mutex).runExclusive(async () => {
                    if (this.action.line.length < this.action_data.line.length) {
                        this.action.line = this.action_data.line
                        return false
                    } else if (this.action.switches.length < this.action_data.switches.length) {
                        this.action.switches = this.action_data.switches
                        return false
                    } else {
                        await this.fetch_next_run()
                        return true
                    }
                }).catch(_ => { })
                if (new_text) {
                    await this.start_type_anime()
                }
            }
        },
        async next_fast() {
            await tryAcquire(this.mutex).runExclusive(async () => {
                await this.fetch_next_run()
                this.action = this.action_data
            }).catch(_ => { })
        },
        async onkeydown(e: KeyboardEvent) {
            if (e.key == "Enter" || e.key == " " || e.key == "ArrowDown") {
                await this.next()
            }
        },
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
    <div class="backboard" v-on:click="next">
        <div class="card card-lines">
            <div class="card-header char">
                <h4 class="card-title">{{ action.character }}</h4>
            </div>
            <div class="card-body lines">
                <p class="h4 card-text">{{ action.line }}</p>
            </div>
        </div>
        <div class="commands">
            <div class="btn-group" role="group">
                <button class="btn btn-outline-primary">
                    <FontAwesomeIcon icon="fas fa-backward-step"></FontAwesomeIcon>
                </button>
                <button class="btn btn-outline-primary">
                    <FontAwesomeIcon icon="fas fa-play"></FontAwesomeIcon>
                </button>
                <button class="btn btn-outline-primary" v-on:click="next">
                    <FontAwesomeIcon icon="fas fa-forward-step"></FontAwesomeIcon>
                </button>
                <button class="btn btn-outline-primary">
                    <FontAwesomeIcon icon="fas fa-forward"></FontAwesomeIcon>
                </button>
                <button class="btn btn-outline-primary">
                    <FontAwesomeIcon icon="fas fa-gear"></FontAwesomeIcon>
                </button>
                <button class="btn btn-outline-primary" v-on:click="go_home">
                    <FontAwesomeIcon icon="fas fa-house"></FontAwesomeIcon>
                </button>
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
    </div>
</template>

<style>
.backboard {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
}

.card-lines {
    position: absolute;
    bottom: 3em;
    width: 100%;
}

.char {
    height: 3em;
}

.lines {
    height: 8em;
}

.commands {
    position: absolute;
    bottom: 0;
    right: 1em;
}

.container-switches {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
    background-color: #00000077;
}

.switches {
    position: absolute;
    width: 100%;
    height: calc(100% - 14em);
}

.switches-center {
    position: absolute;
    width: 100%;
    top: 50%;
    transform: translateY(-50%);
}
</style>
