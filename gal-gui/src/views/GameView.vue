<script setup lang="ts">
import { setTimeout } from 'timers-promises'
import { Mutex, tryAcquire } from 'async-mutex'
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome"
import router from '../router'
import { current_run, next_run, switch_, Action } from '../interop'
</script>

<script lang="ts">
enum ActionState {
    Typing,
    Switching,
    End,
}

export default {
    emits: ["quit"],
    data() {
        return {
            action: { line: "", character: null, switches: [], bg: undefined, bgm: undefined } as Action,
            type_text: "",
            state: ActionState.End,
            mutex: new Mutex(),
        }
    },
    async mounted() {
        document.addEventListener('keydown', this.onkeydown)
        await this.mutex.runExclusive(this.fetch_current_run)
        await this.start_type_anime()
    },
    async unmounted() {
        document.removeEventListener('keydown', this.onkeydown)
    },
    methods: {
        async go_home() {
            await router.replace("/home")
        },
        // Should be called in mutex
        async fetch_current_run() {
            const res = await current_run()
            console.info(res)
            if (res != null) {
                if (res.bg == undefined) {
                    res.bg = this.action.bg
                }
                if (res.bgm == undefined) {
                    res.bgm = this.action.bgm
                }
                const load_new_bgm = (res.bgm != this.action.bgm);
                this.action = res
                if (load_new_bgm) {
                    (document.getElementById("bgm") as HTMLAudioElement).load()
                }
            } else {
                await this.go_home()
            }
        },
        // Should be called in mutex
        async fetch_next_run() {
            await next_run()
            await this.fetch_current_run()
        },
        async switch_run(i: number) {
            await switch_(i)
            await this.mutex.runExclusive(this.fetch_next_run)
            await this.start_type_anime()
        },
        end_typing(wait_switch: boolean = false) {
            this.type_text = this.action.line
            this.state = this.action.switches.length != 0 ? (wait_switch ? this.state : ActionState.Switching) : ActionState.End
        },
        // Shouldn't be called in mutex
        async start_type_anime() {
            this.state = ActionState.Typing
            this.type_text = ""
            while (this.type_text.length < this.action.line.length) {
                this.type_text += this.action.line[this.type_text.length]
                await setTimeout(10)
            }
            this.end_typing(true)
        },
        async next() {
            if (this.state != ActionState.Switching) {
                const new_text = await tryAcquire(this.mutex).runExclusive(async () => {
                    switch (this.state) {
                        case ActionState.Typing:
                            this.end_typing()
                            break
                        case ActionState.End:
                            await this.fetch_next_run()
                            return true
                    }
                    return false
                }).catch(_ => { })
                if (new_text) {
                    await this.start_type_anime()
                }
            }
        },
        async next_fast() {
            if (this.state != ActionState.Switching) {
                await tryAcquire(this.mutex).runExclusive(async () => {
                    await this.fetch_next_run()
                    this.end_typing()
                }).catch(_ => { })
            }
        },
        async onkeydown(e: KeyboardEvent) {
            if (e.key == "Enter" || e.key == " " || e.key == "ArrowDown") {
                await this.next()
            }
        }
    }
}
</script>

<template>
    <audio id="bgm" autoplay hidden>
        <source v-bind:src="action.bgm" type="audio/mpeg">
    </audio>
    <img class="background" v-bind:src="action.bg">
    <div class="card card-lines">
        <div class="card-header char">
            <h4 class="card-title">{{ action.character }}</h4>
        </div>
        <div class="card-body lines">
            <p class="h4 card-text">
                <span v-html="type_text"></span>
            </p>
        </div>
    </div>
    <div class="backboard" v-on:click="next"></div>
    <div class="commands">
        <div class="btn-group" role="group">
            <button class="btn btn-primary btn-command">
                <FontAwesomeIcon icon="fas fa-backward-step"></FontAwesomeIcon>
            </button>
            <button class="btn btn-primary btn-command">
                <FontAwesomeIcon icon="fas fa-play"></FontAwesomeIcon>
            </button>
            <button class="btn btn-primary btn-command" v-on:click="next">
                <FontAwesomeIcon icon="fas fa-forward-step"></FontAwesomeIcon>
            </button>
            <button class="btn btn-primary btn-command">
                <FontAwesomeIcon icon="fas fa-forward"></FontAwesomeIcon>
            </button>
            <button class="btn btn-primary btn-command">
                <FontAwesomeIcon icon="fas fa-gear"></FontAwesomeIcon>
            </button>
            <button class="btn btn-primary btn-command" v-on:click="go_home">
                <FontAwesomeIcon icon="fas fa-house"></FontAwesomeIcon>
            </button>
        </div>
    </div>
    <div class="container-switches" v-bind:hidden="state != ActionState.Switching">
        <div class="switches">
            <div class="switches-center">
                <div class="d-grid gap-5 col-8 mx-auto">
                    <button class="btn btn-primary switch" v-for="(s, i) in action.switches" v-on:click="switch_run(i)"
                        v-bind:disabled="!s.enabled">
                        {{ s.text }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style>
.background {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    max-width: 100%;
    max-height: 100%;
}

.backboard {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 2.5em;
    right: 0;
}

.card-lines {
    position: absolute;
    bottom: 2.5em;
    width: 100%;
    opacity: 0.8;
}

.char {
    height: 3em;
}

.lines {
    height: 8em;
}

/* Copied from .h4 */
.switch {
    font-size: calc(1.275rem + .3vw);
}

.commands {
    position: absolute;
    top: 100%;
    left: 100%;
    transform: translate(-100%, -100%);
}

.btn-command {
    width: 2.5em;
    height: 2.5em;
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
    height: calc(100% - 13.5em);
}

.switches-center {
    position: absolute;
    width: 100%;
    top: 50%;
    transform: translateY(-50%);
}
</style>
