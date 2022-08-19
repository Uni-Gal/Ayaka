<script setup lang="ts">
import * as PIXI from 'pixi.js'
import { Live2DModel } from 'pixi-live2d-display/cubism4'
</script>

<script lang="ts">
export default {
    data() {
        return {}
    },
    async created() {
        const app = new PIXI.Application({
            view: this.$refs.canvas as HTMLCanvasElement
        });

        const model = await Live2DModel.from('shizuku.model.json');

        app.stage.addChild(model);
    }
}
</script>

<template>
    <canvas ref="canvas"></canvas>
</template>
