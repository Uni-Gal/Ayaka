import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import Modal from "vue-bs-modal"
import { createI18n } from 'vue-i18n'
import messages from "./locale"
import { library } from '@fortawesome/fontawesome-svg-core'
import { faForward, faPlay, faGear, faForwardStep, faBackwardStep, faHouse } from '@fortawesome/free-solid-svg-icons'

library.add(faForward, faPlay, faGear, faForwardStep, faBackwardStep, faHouse)

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
