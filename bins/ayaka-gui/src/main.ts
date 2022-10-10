import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { createI18n } from 'vue-i18n'
import messages from "./locale"
import { library } from '@fortawesome/fontawesome-svg-core'
import { faArrowLeft, faBackwardStep, faFileArrowDown, faFileArrowUp, faForward, faForwardStep, faGear, faHouse, faList, faPlay } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { Ticker } from '@pixi/ticker'
import { Live2DModel } from 'pixi-live2d-display'

Live2DModel.registerTicker(Ticker)

library.add(faArrowLeft, faFileArrowUp, faFileArrowDown, faList, faBackwardStep, faPlay, faForwardStep, faForward, faGear, faHouse)

const app = createApp(App)

app.component('font-awesome-icon', FontAwesomeIcon)

app.use(router)

const i18n = createI18n({
    locale: 'en',
    fallbackLocale: 'en',
    messages
})
app.use(i18n)

app.mount('#app')
