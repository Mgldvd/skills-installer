import { createApp } from 'vue'

import './styles/main.scss'
import App from './App.vue'
import { enableKeyboardNavigationFocus } from './utils/keyboardNavigation'

enableKeyboardNavigationFocus(document.body)
createApp(App).mount('#app')
