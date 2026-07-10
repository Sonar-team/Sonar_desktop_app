// Déclarations globales du frontend : bus d'événements `$bus` installé par
// src/main.ts sur les propriétés globales des composants.

/** Bus d'événements applicatif (reset, navigation…), basé sur CustomEvent. */
export interface EventBus {
  emit(event: string, data?: unknown): void;
  on(event: string, callback: (data: any) => void): void;
  off(event: string, callback: (data: any) => void): void;
}

declare module "vue" {
  interface ComponentCustomProperties {
    $bus: EventBus;
  }
}

export {};
