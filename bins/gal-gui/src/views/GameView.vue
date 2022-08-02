<script setup lang="ts">
import { setTimeout } from 'timers-promises'
import { Mutex, tryAcquire } from 'async-mutex'
import ActionCard from '../components/ActionCard.vue'
import IconButton from '../components/IconButton.vue'
import { current_run, next_run, next_back_run, switch_, merge_lines, Action, ActionLineType, ActionLine } from '../interop'
import { cloneDeep } from 'lodash'
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
            action: {
                line: [],
                character: undefined,
                switches: [],
                props: {
                    bg: undefined,
                    bgm: undefined,
                    efm: undefined,
                    voice: undefined,
                    video: undefined,
                },
            } as Action,
            type_text: "",
            type_text_buffer: [] as ActionLine[],
            state: ActionState.End,
            play_state: PlayState.Manual,
            mutex: new Mutex(),
        }
    },
    async mounted() {
        document.addEventListener('keydown', this.onkeydown)
        await this.mutex.runExclusive(this.fetch_current_run)
        this.start_type_anime()
    },
    async unmounted() {
        document.removeEventListener('keydown', this.onkeydown)
    },
    methods: {
        async go_home() {
            await this.$router.replace("/home")
        },
        // Should be called in mutex
        async fetch_current_run() {
            const res = await current_run()
            console.info(res)
            if (res) {
                const load_new_bgm = (res.props.bgm != this.action.props.bgm);
                this.action = res
                if (load_new_bgm) {
                    (this.$refs.bgm as HTMLAudioElement).load()
                }
                if (res.props.efm) {
                    (this.$refs.efm as HTMLAudioElement).load()
                }
                if (res.props.voice) {
                    (this.$refs.voice as HTMLAudioElement).load()
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
        async fetch_next_back_run(): Promise<boolean> {
            const has_back = await next_back_run()
            await this.fetch_current_run()
            return has_back
        },
        end_typing(): boolean {
            this.type_text = merge_lines(this.action.line)
            this.type_text_buffer = []
            if (this.action.switches.length != 0) {
                this.state = ActionState.Switching
                return false
            } else {
                return this.end_switching()
            }
        },
        end_switching(): boolean {
            if (this.action.props.video) {
                this.state = ActionState.Video;
                let element = this.$refs.video as HTMLVideoElement
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
                this.start_type_anime()
            }
        },
        // Shouldn't be called in mutex
        async start_type_anime() {
            this.state = ActionState.Typing
            this.type_text = ""
            this.type_text_buffer = cloneDeep(this.action.line)
            while (this.type_text_buffer.length != 0) {
                if (this.type_text_buffer[0].data.length == 0) {
                    this.type_text_buffer.shift()
                    continue
                }
                switch (ActionLineType[this.type_text_buffer[0].type]) {
                    case ActionLineType.Chars:
                        this.type_text += this.type_text_buffer[0].data[0]
                        this.type_text_buffer[0].data = this.type_text_buffer[0].data.substring(1)
                        await setTimeout(10)
                        break
                    case ActionLineType.Block:
                        this.type_text += this.type_text_buffer[0].data
                        this.type_text_buffer[0].data = ""
                        break
                }
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
                            let element = this.$refs.video as HTMLVideoElement
                            element.pause()
                            this.state = ActionState.End
                        case ActionState.End:
                            return true
                    }
                    return false
                }).catch(_ => { })
                if (new_text) {
                    await this.mutex.runExclusive(this.fetch_next_run)
                    this.start_type_anime()
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
        async next_back() {
            if (this.state != ActionState.Switching) {
                await this.mutex.runExclusive(this.fetch_next_back_run)
                this.start_type_anime()
            }
        },
        async on_history_click() {
            this.$router.push("/history")
        },
        async on_records_click(op: string) {
            await this.$router.push("/records/" + op)
        }
    }
}
</script>

<template>
    <audio ref="bgm" v-bind:src="action.props.bgm" type="audio/mpeg" autoplay hidden loop></audio>
    <audio ref="efm" v-bind:src="action.props.efm" type="audio/mpeg" autoplay hidden></audio>
    <audio ref="voice" v-bind:src="action.props.voice" type="audio/mpeg" autoplay hidden></audio>
    <img class="background" v-bind:src="action.props.bg">
    <div class="card-lines">
        <ActionCard :ch="action.character" :line="type_text"></ActionCard>
    </div>
    <div>
        <h4><span class="badge bg-primary">{{ action.para_title }}</span></h4>
    </div>
    <div class="content-full bg-body" v-bind:hidden="state != ActionState.Video">
        <video ref="video" class="background" v-on:ended="onvideoended" v-bind:src="action.props.video"
            type="video/mp4"></video>
    </div>
    <div class="backboard" v-on:click="next"></div>
    <div class="commands">
        <div class="btn-group" role="group" v-bind:hidden="state == ActionState.Video">
            <IconButton icon="file-arrow-down" @click='on_records_click("save")'></IconButton>
            <IconButton icon="file-arrow-up" @click='on_records_click("load")'></IconButton>
            <IconButton icon="list" @click="on_history_click"></IconButton>
            <IconButton icon="backward-step" @click="next_back"></IconButton>
            <IconButton icon="play" :btnclass='play_state == PlayState.Auto ? "active" : ""'
                @click="on_auto_play_click"></IconButton>
            <IconButton icon="forward-step" @click="next"></IconButton>
            <IconButton icon="forward" :btnclass='play_state == PlayState.FastForward ? "active" : ""'
                @click="on_fast_forward_click"></IconButton>
            <IconButton icon="gear"></IconButton>
            <IconButton icon="house" @click="go_home"></IconButton>
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

.container-switches {
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
