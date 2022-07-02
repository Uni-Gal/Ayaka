import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import Modal from "vue-bs-modal";
import { library } from '@fortawesome/fontawesome-svg-core'
import { faForward, faPlay, faGear, faForwardStep, faBackwardStep, faHouse } from '@fortawesome/free-solid-svg-icons'

library.add(faForward, faPlay, faGear, faForwardStep, faBackwardStep, faHouse)

const app = createApp(App)

app.use(router)
app.use(Modal)

app.mount('#app')
