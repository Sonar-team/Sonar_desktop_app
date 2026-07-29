// Définition pure des routes, séparée de l'historique navigateur afin de
// pouvoir vérifier la navigation avec createMemoryHistory dans les tests.
import type { Component } from "vue";
import type { RouteRecordRaw } from "vue-router";

export const ANALYSE_ROUTE_NAME = "Analyse";

export function createAppRoutes(analyseComponent: Component): RouteRecordRaw[] {
  return [
    {
      path: "/",
      name: ANALYSE_ROUTE_NAME,
      component: analyseComponent,
    },
    {
      path: "/:pathMatch(.*)*",
      redirect: "/",
    },
  ];
}
