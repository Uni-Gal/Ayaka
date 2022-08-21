<script setup lang="ts">
import * as PIXI from 'pixi.js'
import { Live2DModel } from 'pixi-live2d-display'
</script>

<script lang="ts">
export default {
    props: { source: String, scale: Number },
    data() {
        return {
            app: undefined as PIXI.Application | undefined,
        }
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
        if (this.app) {
            this.app.stage.removeChildren(0);
            if (this.source) {
                const model = await Live2DModel.from(this.source);
                this.app.stage.addChild(model)
                this.onresize()
            }
        }
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
                if (this.scale) {
                    canvas_scale *= this.scale
                }
                this.app.stage.children.forEach(c => {
                    c.scale.set(canvas_scale)
                })
            }
        }
    }
}
</script>

<template>
    <canvas class="content-full" ref="canvas"></canvas>
</template>
