<script setup lang="ts">
import { Locale } from 'vue-i18n'
import { locale_native_name, set_locale } from '../interop'
import IconButton from '../components/IconButton.vue';
</script>

<script lang="ts">
export default {
    emits: ["quit"],
    data() {
        return {
            locale_names: new Map<Locale, string>(),
        }
    },
    async created() {
        this.$i18n.availableLocales.forEach(async (locale) => {
            this.locale_names.set(locale, await locale_native_name(locale))
        })
    },
    methods: {
        async on_locale_select(e: Event) {
            await set_locale((e.target as HTMLInputElement).value)
        }
    }
}
</script>

<template>
    <div class="content">
        <div class="d-grid gap-4 col-4 mx-auto">
            <h1>{{ $t("settings") }}</h1>
            <select class="form-select" v-model="$i18n.locale" @change="on_locale_select">
                <option v-for="locale in $i18n.availableLocales" :key="`locale-${locale}`" :value="locale">
                    {{ locale_names.get(locale) ?? locale }}
                </option>
            </select>
        </div>
    </div>
    <div>
        <IconButton icon="arrow-left" @click="$router.back()"></IconButton>
    </div>
</template>
