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
    },
    async updated() {
        if (this.app) {
            this.app.stage.removeChildren(0);
            if (this.source) {
                const model = await Live2DModel.from(this.source);
                this.app.stage.addChild(model)
                model.scale.set(this.scale, this.scale)
            }
        }
    },
    unmounted() {
        this.app?.destroy()
        this.app = undefined
    },
}
</script>

<template>
    <canvas class="content-full" ref="canvas"></canvas>
</template>
