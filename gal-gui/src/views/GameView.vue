<script setup lang="ts">
import { setTimeout } from 'timers-promises'
import { Mutex, tryAcquire } from 'async-mutex'
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome"
import router from '../router'
import { current_run, next_run, switch_, Action, ActionHistoryData, history } from '../interop'
</script>

<script lang="ts">
enum ActionState {
    Typing,
    Typed,
    Switching,
    Video,
    End,
}

enum PlayState {
    Manual,
    Auto,
    FastForward,
}

export default {
    emits: ["quit"],
    data() {
        return {
            action: { line: "", character: undefined, switches: [], bg: undefined, bgm: undefined, video: undefined } as Action,
            type_text: "",
            state: ActionState.End,
            play_state: PlayState.Manual,
            mutex: new Mutex(),
            history: [] as ActionHistoryData[],
            show_history: false
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
            if (res) {
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
        async fetch_next_run(): Promise<boolean> {
            const has_next = await next_run()
            await this.fetch_current_run()
            return has_next
        },
        end_typing(): boolean {
            this.type_text = this.action.line
            if (this.action.switches.length != 0) {
                this.state = ActionState.Switching
                return false
            } else {
                return this.end_switching()
            }
        },
        end_switching(): boolean {
            if (this.action.video != undefined) {
                this.state = ActionState.Video;
                let element = document.getElementById("video") as HTMLVideoElement
                element.load()
                element.play()
                return false
            } else {
                return true
            }
        },
        async switch_run(i: number) {
            await switch_(i)
            if (this.end_switching()) {
                await this.mutex.runExclusive(this.fetch_next_run)
                await this.start_type_anime()
            }
        },
        // Shouldn't be called in mutex
        async start_type_anime() {
            this.state = ActionState.Typing
            this.type_text = ""
            while (this.type_text.length < this.action.line.length) {
                this.type_text += this.action.line[this.type_text.length]
                await setTimeout(10)
            }
            this.state = ActionState.Typed
            if (this.type_text.length == 0) {
                await this.next()
            }
        },
        async next() {
            if (this.state != ActionState.Switching) {
                const new_text = await tryAcquire(this.mutex).runExclusive(async () => {
                    switch (this.state) {
                        case ActionState.Typing:
                        case ActionState.Typed:
                            return this.end_typing()
                        case ActionState.Video:
                            let element = document.getElementById("video") as HTMLVideoElement
                            element.pause()
                            this.state = ActionState.End
                        case ActionState.End:
                            return true
                    }
                    return false
                }).catch(_ => { })
                if (new_text) {
                    await this.mutex.runExclusive(this.fetch_next_run)
                    await this.start_type_anime()
                }
            }
        },
        async on_auto_play_click() {
            if (this.play_state != PlayState.Auto) {
                this.play_state = PlayState.Auto
                this.end_typing()
                while (this.play_state == PlayState.Auto && (this.state != ActionState.Switching && this.state != ActionState.Video)) {
                    const has_next = await tryAcquire(this.mutex).runExclusive(async () => {
                        const has_next = await this.fetch_next_run()
                        await this.start_type_anime()
                        this.end_typing()
                        return has_next
                    }).catch(_ => { })
                    if (!has_next) {
                        break
                    }
                }
            }
            this.play_state = PlayState.Manual
        },
        async on_fast_forward_click() {
            if (this.play_state != PlayState.FastForward) {
                this.play_state = PlayState.FastForward
                this.end_typing()
                while (this.play_state == PlayState.FastForward && (this.state != ActionState.Switching && this.state != ActionState.Video)) {
                    await setTimeout(20)
                    const has_next = await tryAcquire(this.mutex).runExclusive(async () => {
                        const has_next = await this.fetch_next_run()
                        this.end_typing()
                        return has_next
                    }).catch(_ => { })
                    if (!has_next) {
                        break
                    }
                }
            }
            this.play_state = PlayState.Manual
        },
        async onkeydown(e: KeyboardEvent) {
            if (e.key == "Enter" || e.key == " " || e.key == "ArrowDown") {
                await this.next()
            }
        },
        async onvideoended() {
            this.state = ActionState.End
            await this.next()
        },
        async on_history_click() {
            if (!this.show_history) {
                this.history = await history()
                this.show_history = true
            } else {
                this.show_history = false
            }
        }
    }
}
</script>

<template>
    <audio id="bgm" autoplay hidden>
        <source v-bind:src="action.bgm" type="audio/mpeg" />
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
    <div class="content-full bg-body" v-bind:hidden="state != ActionState.Video">
        <video id="video" class="background" v-on:ended="onvideoended">
            <source v-bind:src="action.video" type="video/mp4" />
        </video>
    </div>
    <div class="backboard" v-on:click="next"></div>
    <div class="commands">
        <div class="btn-group" role="group" v-bind:hidden="state == ActionState.Video">
            <button class="btn btn-primary btn-command">
                <FontAwesomeIcon icon="fas fa-file-arrow-down"></FontAwesomeIcon>
            </button>
            <button class="btn btn-primary btn-command">
                <FontAwesomeIcon icon="fas fa-file-arrow-up"></FontAwesomeIcon>
            </button>
            <button class="btn btn-primary btn-command" v-on:click="on_history_click">
                <FontAwesomeIcon icon="fas fa-list"></FontAwesomeIcon>
            </button>
            <button v-bind:class='`btn btn-primary btn-command ${play_state == PlayState.Auto ? "active" : ""}`'
                v-on:click="on_auto_play_click">
                <FontAwesomeIcon icon="fas fa-play"></FontAwesomeIcon>
            </button>
            <button class="btn btn-primary btn-command" v-on:click="next">
                <FontAwesomeIcon icon="fas fa-forward-step"></FontAwesomeIcon>
            </button>
            <button v-bind:class='`btn btn-primary btn-command ${play_state == PlayState.FastForward ? "active" : ""}`'
                v-on:click="on_fast_forward_click">
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
    <div class="content-full container-switches" v-bind:hidden="state != ActionState.Switching">
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
    <div id="history" class="content-full container-history" v-bind:hidden="!show_history"
        v-on:click="on_history_click">
        <ul class="list-group">
            <li class="list-group-item" v-for="h in history">
                <div class="card">
                    <div class="card-header char">
                        <h4 class="card-title">{{ h.character }}</h4>
                    </div>
                    <div class="card-body lines">
                        <p class="h4 card-text">
                            <span v-html="h.line"></span>
                        </p>
                    </div>
                </div>
            </li>
        </ul>
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
    background-color: #00000077;
}

.container-history {
    overflow-y: scroll;
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
