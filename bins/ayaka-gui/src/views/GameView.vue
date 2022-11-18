<script setup lang="ts">
import { setTimeout } from 'timers-promises'
import { Mutex, tryAcquire } from 'async-mutex'
import ActionCard from '../components/ActionCard.vue'
import IconButton from '../components/IconButton.vue'
import { conv_src, current_run, current_action, current_title, next_run, next_back_run, switch_, merge_lines, RawContext, ActionType, ActionText, CustomVars, Switch, ActionLineType, ActionLine, current_visited } from '../interop'
import { cloneDeep } from 'lodash'
import Live2D from '../components/Live2D.vue'
import { Modal } from 'bootstrap'
</script>

<script lang="ts">
enum PlayState {
    Manual,
    Auto,
    FastForward,
}

function wait_play(e: HTMLAudioElement): Promise<void> {
    return new Promise<void>((resolve, _) => {
        e.addEventListener("ended", () => { resolve() }, { once: true })
    })
}

function live2d_names(locals: { ch_models?: string }): string[] {
    return (locals.ch_models ?? "").split(",").filter(s => s.length != 0)
}

export default {
    emits: ["quit"],
    data() {
        return {
            raw_ctx: {
                cur_para: "", cur_act: 0, history: [], locals: {}
            } as RawContext,
            action: {
                text: [], vars: {}
            } as ActionText,
            sub_action_text: [] as ActionLine[],
            switches: [] as Switch[],
            vars: {} as CustomVars,
            title: "",
            type_text: "",
            type_text_buffer: [] as ActionLine[],
            type_sub_text: "",
            type_sub_text_buffer: [] as ActionLine[],
            bg: undefined as string | undefined,
            bgm: undefined as string | undefined,
            voice: undefined as string | undefined,
            video: undefined as string | undefined,
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
        go_home() {
            let modal = new Modal(this.$refs.homeModal as HTMLElement)
            modal.show()
        },
        async go_home_direct() {
            await this.$router.replace("/home")
        },
        // Should be called in mutex
        async fetch_current_run() {
            const ctx = await current_run()
            this.bg = await conv_src(ctx?.locals.bg)
            this.bgm = await conv_src(ctx?.locals.bgm)
            const actions = await current_action()
            this.title = await current_title() ?? ""
            console.info(actions)
            if (ctx && actions) {
                this.raw_ctx = ctx
                let [action, sub_action] = actions
                switch (ActionType[action.type]) {
                    case ActionType.Empty:
                        this.action = { text: [], vars: {} } as ActionText
                        this.sub_action_text = []
                        this.switches = []
                        this.vars = {}
                        break
                    case ActionType.Text:
                        this.action = action.data as ActionText
                        this.sub_action_text = (sub_action?.data as ActionText | undefined)?.text ?? []
                        this.switches = []
                        this.vars = {}
                        this.start_type_anime(true)
                        break
                    case ActionType.Switches:
                        this.play_state = PlayState.Manual
                        this.switches = action.data as Switch[]
                        this.vars = {}
                        break
                    case ActionType.Custom:
                        this.action = { text: [], vars: {} } as ActionText
                        this.sub_action_text = []
                        this.switches = []
                        let data = action.data as CustomVars
                        this.vars = data
                        if (data.video) {
                            this.play_state = PlayState.Manual
                        }
                        break
                }
                this.voice = await conv_src(this.action.vars.voice)
                this.video = await conv_src(this.vars.video)
            }
        },
        // Should be called in mutex
        async fetch_next_run() {
            const has_next = await next_run()
            if (!has_next) {
                this.play_state = PlayState.Manual
                await this.go_home_direct()
            }
            await this.fetch_current_run()
        },
        async fetch_next_back_run() {
            await next_back_run()
            await this.fetch_current_run()
        },
        end_typing() {
            this.type_text = merge_lines(this.action.text)
            this.type_text_buffer = []
        },
        async switch_run(i: number) {
            await switch_(i)
            await this.mutex.runExclusive(this.fetch_next_run)
        },
        async type_anime_impl() {
            this.type_text = ""
            this.type_text_buffer = cloneDeep(this.action.text)
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
        },
        async sub_type_anime_impl() {
            this.type_sub_text = ""
            this.type_sub_text_buffer = cloneDeep(this.sub_action_text)
            while (this.type_sub_text_buffer.length != 0) {
                if (this.type_sub_text_buffer[0].data.length == 0) {
                    this.type_sub_text_buffer.shift()
                    continue
                }
                switch (ActionLineType[this.type_sub_text_buffer[0].type]) {
                    case ActionLineType.Chars:
                        this.type_sub_text += this.type_sub_text_buffer[0].data[0]
                        this.type_sub_text_buffer[0].data = this.type_sub_text_buffer[0].data.substring(1)
                        await setTimeout(10)
                        break
                    case ActionLineType.Block:
                        this.type_sub_text += this.type_sub_text_buffer[0].data
                        this.type_sub_text_buffer[0].data = ""
                        break
                }
            }
        },
        // Shouldn't be called in mutex
        async start_type_anime(timeout: boolean = false) {
            let values = timeout ? [setTimeout(3000)] : []
            if (this.action.vars.voice) {
                let voice = this.$refs.voice as HTMLAudioElement
                values.push(wait_play(voice))
            }
            values.push(this.type_anime_impl(), this.sub_type_anime_impl())
            await Promise.all(values)
        },
        async next() {
            await tryAcquire(this.mutex).runExclusive(this.fetch_next_run).catch(_ => { });
        },
        async on_auto_play_click() {
            if (this.play_state != PlayState.Auto) {
                this.play_state = PlayState.Auto
                this.end_typing()
                while (this.play_state == PlayState.Auto) {
                    await tryAcquire(this.mutex).runExclusive(async () => {
                        await this.fetch_next_run()
                        await this.start_type_anime(true)
                        this.end_typing()
                    }).catch(_ => { })
                }
            }
            this.play_state = PlayState.Manual
        },
        async on_fast_forward_click() {
            if (this.play_state != PlayState.FastForward) {
                this.play_state = PlayState.FastForward
                this.end_typing()
                while (this.play_state == PlayState.FastForward) {
                    await setTimeout(20)
                    await tryAcquire(this.mutex).runExclusive(async () => {
                        await this.fetch_next_run()
                        this.end_typing()
                    }).catch(_ => { })
                    if (!await current_visited()) {
                        break
                    }
                }
            }
            this.play_state = PlayState.Manual
        },
        async onkeydown(e: KeyboardEvent) {
            switch (e.key) {
                case "Enter":
                case " ":
                case "ArrowDown":
                    await this.next()
                    break
                case "ArrowUp":
                    await this.next_back()
                    break
            }
        },
        async onvideoended() {
            await this.next()
        },
        async next_back() {
            await this.mutex.runExclusive(this.fetch_next_back_run)
            this.start_type_anime()
        },
        async on_history_click() {
            await this.$router.push("/history")
        },
        async on_records_click(op: string) {
            await this.$router.push("/records/" + op)
        },
        async on_settings_click() {
            await this.$router.push("/settings")
        }
    }
}
</script>

<template>
    <audio ref="bgm" :src="bgm" type="audio/mpeg" autoplay hidden loop></audio>
    <audio ref="voice" :src="voice" type="audio/mpeg" autoplay hidden></audio>
    <img class="background" :src="bg">
    <Live2D :names="live2d_names(raw_ctx.locals)"></Live2D>
    <div class="card-lines">
        <ActionCard :ch="action.character" :line="type_text" :sub_line="type_sub_text"></ActionCard>
    </div>
    <div>
        <h4><span class="badge bg-primary">{{ title }}</span></h4>
    </div>
    <div class="logo d-flex align-items-center">
        <span>Powered by Ayaka.</span>
    </div>
    <div class="content-full bg-body" :hidden="!vars.video">
        <video ref="video" class="background" @ended="onvideoended" :src="video" type="video/mp4" autoplay></video>
    </div>
    <div class="backboard" @click="next"></div>
    <div class="commands">
        <div class="btn-group" role="group">
            <IconButton icon="file-arrow-down" @click='on_records_click("save")'></IconButton>
            <IconButton icon="file-arrow-up" @click='on_records_click("load")'></IconButton>
            <IconButton icon="list" @click="on_history_click"></IconButton>
            <IconButton icon="backward-step" @click="next_back"></IconButton>
            <IconButton icon="play" :btnclass='play_state == PlayState.Auto ? "active" : ""'
                @click="on_auto_play_click"></IconButton>
            <IconButton icon="forward-step" @click="next"></IconButton>
            <IconButton icon="forward" :btnclass='play_state == PlayState.FastForward ? "active" : ""'
                @click="on_fast_forward_click"></IconButton>
            <IconButton icon="gear" @click="on_settings_click"></IconButton>
            <IconButton icon="house" @click="go_home"></IconButton>
        </div>
    </div>
    <div class="content-full container-switches" :hidden="switches.length == 0">
        <div class="switches">
            <div class="switches-center">
                <div class="d-grid gap-5 col-8 mx-auto">
                    <button class="btn btn-primary switch" v-for="(s, i) in switches" @click="switch_run(i)"
                        :disabled="!s.enabled">
                        {{ s.text }}
                    </button>
                </div>
            </div>
        </div>
    </div>

    <div class="modal fade" ref="homeModal" tabindex="-1">
        <div class="modal-dialog">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">{{ $t("goHome") }}</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">{{ $t("goHomeConfirm") }}</div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-primary" data-bs-dismiss="modal">
                        {{ $t("dialogNo") }}
                    </button>
                    <button type="button" class="btn btn-secondary" data-bs-dismiss="modal" @click="go_home_direct">
                        {{ $t("dialogYes") }}
                    </button>
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

.logo {
    position: absolute;
    top: 100%;
    left: 0;
    height: 2.5em;
    transform: translateY(-100%);
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
