<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
import { appWindow } from "@tauri-apps/api/window";
import { assert } from 'console';
import { Locale } from 'vue-i18n';
import router from '../router';
</script>

<script lang="ts">
export default {
    data() {
        return {
            title: ""
        }
    },
    async created() {
        appWindow.listen("tauri://close-requested", async () => { await this.quit() })
        const loc = localStorage.getItem("locale") ?? await invoke<string | null>("choose_locale", { locales: this.$i18n.availableLocales })
        if (loc != null) {
            if (this.$i18n.availableLocales.includes(loc)) {
                this.$i18n.locale = loc
                this.save_locale(loc)
            } else {
                console.error("Wrong locale %s", loc)
            }
        }
        const res = await invoke<{ title: string }>("info")
        this.title = res.title
    },
    methods: {
        async new_game() {
            await invoke<void>("start_new", { locale: this.$i18n.locale })
            if (await invoke<boolean>("next_run")) {
                router.push("/game")
            }
        },
        async quit() {
            const confirmed = await this.$vbsModal.confirm({
                title: this.$t("quit"),
                message: this.$t("quitConfirm"),
            })
            if (confirmed) {
                await appWindow.close()
            }
        },
        save_locale(loc: Locale) {
            localStorage.setItem("locale", loc)
        }
    }
}
</script>

<template>
    <div class="content">
        <div class="d-grid gap-4 col-4 mx-auto">
            <h1>{{ title }}</h1>
            <button class="btn btn-primary" v-on:click="new_game">{{ $t("newGame") }}</button>
            <router-link class="btn btn-primary" to="/about">{{ $t("about") }}</router-link>
            <button class="btn btn-primary" v-on:click="quit">{{ $t("quit") }}</button>
            <select class="form-select" v-model="$i18n.locale"
                v-on:change="(e) => save_locale((e.target as HTMLInputElement).value)">
                <option v-for="locale in $i18n.availableLocales" :key="`locale-${locale}`" :value="locale">{{ locale
                }}</option>
            </select>
        </div>
    </div>
</template>
