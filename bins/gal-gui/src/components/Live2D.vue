<script setup lang="ts">
import * as PIXI from 'pixi.js'
import { Live2DModel } from 'pixi-live2d-display/cubism4'
</script>

<script lang="ts">
export default {
    props: { source: String },
    data() {
        return {
            app: undefined as PIXI.Application | undefined,
            model: undefined as PIXI.DisplayObject | undefined,
        }
    },
    async mounted() {
        if (this.source) {
            const app = new PIXI.Application({
                view: this.$refs.canvas as HTMLCanvasElement,
                backgroundAlpha: 0,
                resizeTo: this.$refs.canvas as HTMLElement,
            });
            this.app = app

            const source = decodeURIComponent(this.source)
            const model = await Live2DModel.from(source);

            app.stage.addChild(model);
            this.model = model

            model.scale.set(0.3);
        }
    },
    async unmounted() {
        if (this.app) {
            if (this.model) {
                this.app.stage.removeChild(this.model as PIXI.DisplayObject)
                this.model = undefined
            }
            this.app = undefined
        }
    }
}
</script>

<template>
    <canvas class="content-full" ref="canvas"></canvas>
</template>
