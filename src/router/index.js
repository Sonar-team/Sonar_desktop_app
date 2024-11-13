import { createRouter, createWebHistory } from 'vue-router'
import HomeComponent from '../views/homeView.vue'; // Corrected path for App.vue
import AnalyseView from '../views/analyseView.vue'; // Corrected path for AnalyseView.vue
import ReadPcapView from '../views/readPcapView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
  {
    path: '/', // URL path for HomeComponent
    name: 'home',
    component: HomeComponent
  },
  {
    path: '/analyse/:netInterface/:confidentialite/:installationName/:time/:currentTime',
    name: 'Analyse',
    component: AnalyseView,
    props: true
  },
  {
    path: '/readPcap',
      name: 'ReadPcap',
      component: ReadPcapView,
      props: (route) => ({ pcapList: JSON.parse(route.query.pcapList || '[]') })
    },
]
})

export default router
