import Vue from 'vue'
import App from './App.vue'
import VueRouter from 'vue-router'
import axios from "axios"
import router from './router.js'
import ElementUI from 'element-ui';
import 'element-ui/lib/theme-chalk/index.css';

axios.defaults.baseURL = process.env.VUE_APP_PUBLIC_URL

Vue.use(VueRouter)
Vue.use(ElementUI)
Vue.prototype.$axios = axios
Vue.config.productionTip = false

new Vue({
  router,
  render: h => h(App),
}).$mount('#app')
