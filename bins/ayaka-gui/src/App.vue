<script setup lang="ts">
import 'bootstrap-dark-5/dist/css/bootstrap-dark.min.css'
import { appWindow } from "@tauri-apps/api/window"
import { init, save_all } from './interop'
import { Modal } from 'bootstrap'
</script>

<script lang="ts">
export default {
    async created() {
        await init()
        appWindow.listen("tauri://close-requested", this.quit)
    },
    methods: {
        quit() {
            let modal = new Modal(this.$refs.quitModal as HTMLElement)
            modal.show()
        },
        async quit_direct() {
            await save_all()
            await appWindow.close()
        }
    }
}
</script>

<template>
    <router-view @quit="quit" />

    <div class="modal fade" ref="quitModal" tabindex="-1">
        <div class="modal-dialog">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">{{ $t("quit") }}</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">{{ $t("quitConfirm") }}</div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-primary" data-bs-dismiss="modal">
                        {{ $t("dialogNo") }}
                    </button>
                    <button type="button" class="btn btn-secondary" @click="quit_direct">
                        {{ $t("dialogYes") }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style>
@import './assets/base.css';
</style>
