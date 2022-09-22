<script setup lang="ts">
import { Locale } from 'vue-i18n'
import { set_locale, set_sub_locale, avaliable_locale, get_settings } from '../interop'
import IconButton from '../components/IconButton.vue';
</script>

<script lang="ts">
function locale_native_name(loc: Locale): string {
    return new Intl.DisplayNames(loc, { type: "language" }).of(loc) ?? ""
}

export default {
    emits: ["quit"],
    data() {
        return {
            locales: [] as Locale[],
            sub_locale: "none"
        }
    },
    async created() {
        this.locales = await avaliable_locale(this.$i18n.availableLocales)
        let sub_locale = (await get_settings())?.sub_lang
        if (sub_locale) {
            this.sub_locale = sub_locale
        }
    },
    methods: {
        async on_locale_select(e: Event) {
            await set_locale((e.target as HTMLInputElement).value)
        },
        async on_sub_locale_select(e: Event) {
            let loc = (e.target as HTMLInputElement).value
            await set_sub_locale(loc == "none" ? undefined : loc)
        }
    }
}
</script>

<template>
    <div class="content">
        <div class="d-grid gap-4 col-4 mx-auto">
            <h1>{{ $t("settings") }}</h1>
            <h2>{{ $t("language") }}</h2>
            <select class="form-select" v-model="$i18n.locale" @change="on_locale_select">
                <option v-for="locale in locales" :value="locale">
                    {{ locale_native_name(locale) }}
                </option>
            </select>
            <h2>{{ $t("subLanguage") }}</h2>
            <select class="form-select" v-model="sub_locale" @change="on_sub_locale_select">
                <option value="none">{{ $t("none") }}</option>
                <option v-for="locale in locales" :value="locale">
                    {{ locale_native_name(locale) }}
                </option>
            </select>
        </div>
    </div>
    <div>
        <IconButton icon="arrow-left" @click="$router.back()"></IconButton>
    </div>
</template>
