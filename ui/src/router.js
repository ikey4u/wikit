import VueRouter from 'vue-router'
import Dictionary from './components/Dictionary.vue'
import Tools from './components/Tools.vue'
import About from './components/About.vue'

const routes = [
  {
      path: '/',
      component: Dictionary,
  },
  {
    path: '/tools',
    component: Tools,
  },
  {
    path: '/about',
    component: About,
  },
];

export default new VueRouter({
  mode: 'history',
  base: '/',
  routes,
});
