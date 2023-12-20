import { createRouter, createWebHistory } from 'vue-router'
import SetInfosView from '../views/SetInfosView.vue'; // Corrected path for App.vue
import AnalyseView from '../views/analyseView.vue'; // Corrected path for AnalyseView.vue


const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
  {
    path: '/', // URL path for HomeComponent
    name: 'SetInfosView',
    component: SetInfosView
  },
  {
    path: '/analyse/:netInterface/:confidentialite/:installationName/:time',
    name: 'Analyse',
    component: AnalyseView,
    props: true
  },

]
})

export default router
