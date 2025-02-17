// Importer les outils de test
import { createLocalVue, shallowMount } from "@vue/test-utils";

import MyComponent from "@/src/views/analyseView.vue"; // Mettez à jour le chemin d'accès en fonction de votre structure de fichiers

// Mock des composants enfants et des dépendances
jest.mock("@/components/NavBar/SideBar.vue", () => ({
  name: "Sidebar",
  render: (h) => h("div"),
}));
jest.mock("@/components/CaptureVue/BottomLong.vue", () => ({
  name: "BottomLong",
  render: (h) => h("div"),
}));
jest.mock("@/components/CaptureVue/Matrice.vue", () => ({
  name: "Matrice",
  render: (h) => h("div"),
}));
jest.mock("@/components/AnalyseVue/GraphVue/NetworkGraphComponent.vue", () => ({
  name: "NetworkGraphComponent",
  render: (h) => h("div"),
}));
jest.mock("@tauri-apps/api/tauri", () => ({
  invoke: jest.fn(),
}));

describe("MyComponent.vue", () => {
  let wrapper;

  beforeEach(() => {
    const localVue = createLocalVue();
    localVue.prototype.$route = {
      params: {/* Simuler les params du routeur ici */},
    };
    localVue.prototype.$bus = { on: jest.fn(), off: jest.fn() }; // Simuler l'event bus
    wrapper = shallowMount(MyComponent, { localVue });
  });

  it("affiche le bon titre basé sur les props", () => {
    // Ajouter des assertions pour vérifier le contenu du titre
  });

  it("montre Matrice lorsque showMatrice est true", () => {
    expect(wrapper.findComponent({ name: "Matrice" }).exists()).toBe(true);
    expect(wrapper.findComponent({ name: "NetworkGraphComponent" }).exists())
      .toBe(false);
  });

  it("montre NetworkGraphComponent lorsque showMatrice est false", async () => {
    await wrapper.setData({ showMatrice: false });
    expect(wrapper.findComponent({ name: "NetworkGraphComponent" }).exists())
      .toBe(true);
    expect(wrapper.findComponent({ name: "Matrice" }).exists()).toBe(false);
  });

  // Ajouter plus de tests selon les besoins
});
