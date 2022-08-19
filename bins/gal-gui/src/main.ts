import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import Modal from "vue-bs-modal"
import { createI18n } from 'vue-i18n'
import messages from "./locale"
import { library } from '@fortawesome/fontawesome-svg-core'
import { fas } from '@fortawesome/free-solid-svg-icons'
import { Ticker } from '@pixi/ticker'
import { Live2DModel } from 'pixi-live2d-display/cubism4'

Live2DModel.registerTicker(Ticker)

library.add(fas)

const app = createApp(App)

app.use(router)
app.use(Modal)

const i18n = createI18n({
    locale: 'en',
    fallbackLocale: 'en',
    messages
})
app.use(i18n)

app.mount('#app')
