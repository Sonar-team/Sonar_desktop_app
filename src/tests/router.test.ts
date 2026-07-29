import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { createMemoryHistory, createRouter } from "vue-router";

import { ANALYSE_ROUTE_NAME, createAppRoutes } from "../router/routes.ts";

function testRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: createAppRoutes({ template: "<main>Analyse</main>" }),
  });
}

Deno.test("router - la vue d'analyse est l'unique route publique", () => {
  const routes = createAppRoutes({ template: "<main>Analyse</main>" });

  assertEquals(
    routes.map(({ path, name, redirect }) => ({ path, name, redirect })),
    [
      { path: "/", name: ANALYSE_ROUTE_NAME, redirect: undefined },
      {
        path: "/:pathMatch(.*)*",
        name: undefined,
        redirect: "/",
      },
    ],
  );
});

Deno.test("router - une URL inconnue revient vers l'analyse", async () => {
  const router = testRouter();

  await router.push("/route-inconnue");
  await router.isReady();

  assertEquals(router.currentRoute.value.name, ANALYSE_ROUTE_NAME);
  assertEquals(router.currentRoute.value.fullPath, "/");
});
