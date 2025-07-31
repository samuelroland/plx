import { createApp } from "vue";
import App from "./App.vue";
import Notifications from "notiwind";
import "./main.css";
import { createPinia } from "pinia";

const pinia = createPinia();
const app = createApp(App);

app.use(pinia);
app.use(Notifications);
app.mount("#app");
