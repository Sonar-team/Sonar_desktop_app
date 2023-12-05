import { createRouter, createWebHistory } from 'vue-router'
import HomeComponent from '../views/homeView.vue'; // Corrected path for App.vue
import AnalyseView from '../views/analyseView.vue'; // Corrected path for AnalyseView.vue
import GraphView from '../views/graphView.vue'; // Corrected path for

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
  {
    path: '/', // URL path for HomeComponent
    name: 'home',
    component: HomeComponent
  },
  {
    path: '/analyse', // URL path for AnalyseView
    name: 'analyseView',
    component: AnalyseView
  },
  {
    path: '/graph', // URL path for AnalyseView
    name: 'graphView',
    component: GraphView
  },
]
})

export default router
