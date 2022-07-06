<script setup lang="ts">
import { Locale } from 'vue-i18n'
import { info, next_run, start_new, locale_native_name } from '../interop'
import router from '../router'
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            title: "",
            locale_names: new Map<Locale, string>()
        }
    },
    async created() {
        const res = await info()
        this.title = res.title
        this.$i18n.availableLocales.forEach(async (locale) => {
            this.locale_names.set(locale, await this.locale_native_name(locale))
        })
    },
    methods: {
        async new_game() {
            await start_new(this.$i18n.locale)
            if (await next_run()) {
                router.replace("/game")
            }
        },
        save_locale(loc: Locale) {
            localStorage.setItem("locale", loc)
        },
        async locale_native_name(loc: Locale) {
            return await locale_native_name(loc)
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
            <button class="btn btn-primary" v-on:click="$emit('quit')">{{ $t("quit") }}</button>
            <select class="form-select" v-model="$i18n.locale"
                v-on:change="(e) => save_locale((e.target as HTMLInputElement).value)">
                <option v-for="locale in $i18n.availableLocales" :key="`locale-${locale}`" :value="locale">
                    {{ locale_names.get(locale) ?? locale }}
                </option>
            </select>
        </div>
    </div>
</template>
