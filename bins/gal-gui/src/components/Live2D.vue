<script setup lang="ts">
import * as PIXI from 'pixi.js'
import { Live2DModel } from 'pixi-live2d-display'
import { Mutex } from 'async-mutex'
import { GameInfo, info } from '../interop';
</script>

<script lang="ts">
export default {
    props: { sources: Array<string>, names: Array<string> },
    data() {
        return {
            game: {} as GameInfo,
            app: undefined as PIXI.Application | undefined,
            mutex: new Mutex(),
        }
    },
    async created() {
        this.game = await info()
    },
    async mounted() {
        this.app = new PIXI.Application({
            view: this.$refs.canvas as HTMLCanvasElement,
            backgroundAlpha: 0,
            resizeTo: this.$refs.canvas as HTMLElement,
        })
        window.addEventListener("resize", this.onresize)
    },
    async updated() {
        await this.mutex.runExclusive(async () => {
            if (this.app) {
                this.app.stage.removeChildren(0)
                if (this.sources) {
                    for (const [s, name] of this.sources.map((s, i) => [s, this.names?.at(i) ?? ""])) {
                        let model = await Live2DModel.from(s)
                        model.name = name
                        this.app.stage.addChild(model)
                    }
                    this.onresize()
                }
            }
        })
    },
    unmounted() {
        window.removeEventListener("resize", this.onresize)
        this.app?.destroy()
        this.app = undefined
    },
    methods: {
        onresize() {
            if (this.app) {
                let canvas_scale = window.innerHeight / 600.0
                const count = this.app.stage.children.length
                const width_per_ch = this.app.view.clientWidth / count
                this.app.stage.children.forEach((c, i) => {
                    let m = c as Live2DModel;
                    m.x = width_per_ch * (i as number) + width_per_ch / 2
                    m.anchor.set(0.5, 0)
                    m.scale.set(canvas_scale * this.model_scale(m.name))
                })
            }
        },
        model_scale(key?: string): number {
            if (key) {
                return parseFloat((this.game.props as any)["ch_" + key + "_scale"])
            }
            return 1
        }
    }
}
</script>

<template>
    <canvas class="content-full" ref="canvas"></canvas>
</template>
