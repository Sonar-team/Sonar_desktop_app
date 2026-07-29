// Router de production : le seul parcours public est l'analyse. Les anciennes
// routes d'import ont été supprimées au profit d'ImportPanel (#145).
import { createRouter, createWebHistory } from "vue-router";

import AnalyseView from "../views/analyseView.vue";
import { createAppRoutes } from "./routes";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: createAppRoutes(AnalyseView),
});

export default router;
