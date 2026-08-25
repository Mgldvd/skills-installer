import { createApp } from "vue";

import "../styles/main.scss";
import { enableKeyboardNavigationFocus } from "../utils/keyboardNavigation";
import StyleGuideView from "./StyleGuideView.vue";

enableKeyboardNavigationFocus(document.body);
createApp(StyleGuideView).mount("#design-app");
