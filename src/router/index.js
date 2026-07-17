// Routes de l'application : vue d'analyse (par défaut), accueil capture et
// lecture de PCAP.
import { createRouter, createWebHistory } from "vue-router";
import AnalyseView from "../views/analyseView.vue";
import ReadPcapView from "../views/readPcapView.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/", // URL path for HomeComponent
      name: "Analyse",
      component: AnalyseView,
    },

    {
      path: "/readPcap",
      name: "ReadPcap",
      component: ReadPcapView,
      props: (route) => ({
        pcapList: JSON.parse(route.query.pcapList || "[]"),
      }),
    },
  ],
});

export default router;
