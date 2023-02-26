<script setup lang="ts">
import { Locale } from 'vue-i18n'
import { set_locale, set_sub_locale, set_volumes, avaliable_locale, get_settings } from '../interop'
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
            sub_locale: "none",
            bgm_volume: "100",
            voice_volume: "100",
            video_volume: "100"
        }
    },
    async created() {
        this.locales = await avaliable_locale(this.$i18n.availableLocales)
        const settings = await get_settings()
        let sub_locale = settings.sub_lang
        if (sub_locale && this.$i18n.locale != sub_locale) {
            this.sub_locale = sub_locale
        }
        this.bgm_volume = settings.bgm_volume.toString()
        this.voice_volume = settings.voice_volume.toString()
        this.video_volume = settings.video_volume.toString()
    },
    methods: {
        async on_locale_select(e: Event) {
            let loc = (e.target as HTMLInputElement).value
            if (loc == this.sub_locale) {
                this.sub_locale = "none"
            }
            await set_locale((e.target as HTMLInputElement).value)
        },
        async on_sub_locale_select(e: Event) {
            let loc = (e.target as HTMLInputElement).value
            if (loc == this.$i18n.locale) {
                this.sub_locale = loc = "none"
            }
            await set_sub_locale(loc == "none" ? undefined : loc)
        },
        async on_volume_change() {
            await set_volumes(parseInt(this.bgm_volume), parseInt(this.voice_volume), parseInt(this.video_volume))
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
            <h2>BGM volume: {{ bgm_volume }}</h2>
            <div class="range">
                <input type="range" class="form-range" min="0" max="100" step="1" v-model="bgm_volume"
                    @input="on_volume_change">
            </div>
            <h2>Voice volume: {{ voice_volume }}</h2>
            <div class="range">
                <input type="range" class="form-range" min="0" max="100" step="1" v-model="voice_volume"
                    @input="on_volume_change">
            </div>
            <h2>Video volume: {{ video_volume }}</h2>
            <div class="range">
                <input type="range" class="form-range" min="0" max="100" step="1" v-model="video_volume"
                    @input="on_volume_change">
            </div>
        </div>
    </div>
    <div>
        <IconButton icon="arrow-left" @click="$router.back()"></IconButton>
    </div>
</template>
