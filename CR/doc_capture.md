# Documentation du Module de Capture de Paquets

## Vue d'ensemble

Le module de capture de paquets est une composante essentielle de l'application
Sonar, permettant la capture en temps réel du trafic réseau. Il est construit en
Rust et utilise la bibliothèque `pcap` pour une capture efficace des paquets.

## Architecture

### Composants principaux

1. **CaptureHandle**
   - Gère le cycle de vie de la capture
   - Coordonne les différents threads
   - Gère les événements de capture

2. **Threads**
   - **Capture** : Capture les paquets réseau
   - **Traitement** : Analyse et traite les paquets capturés

3. **Buffer Pool**
   - Gestion efficace de la mémoire pour les paquets
   - Réduction des allocations/désallocations

## Fonctionnalités clés

### 1. Démarrer une capture

```rust
pub fn start_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    app: AppHandle,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<CaptureStatus, CaptureStateError>
```

### 2. Arrêter une capture

```rust
pub fn stop_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<CaptureStatus, CaptureStateError>
```

### 3. Configuration

- Interface réseau
- Taille du buffer
- Taille maximale des paquets
- Filtres BPF

## Gestion des événements

### Types d'événements

- `Started` : Démarrage de la capture
- `Stats` : Statistiques de capture
- `Packet` : Paquet capturé
- `Graph` : Mise à jour du graphe réseau
- `Finished` : Fin de la capture

## Bonnes pratiques

### Gestion de la mémoire

- Utilisation d'un pool de buffers pour éviter les allocations fréquentes
- Gestion propre des ressources avec `Drop` pour les buffers

### Performance

- Capture non-bloquante
- Traitement asynchrone
- Gestion de la pression mémoire

## Exemple d'utilisation

```rust
// Configuration de la capture
let config = CaptureConfig {
    device_name: "eth0".to_string(),
    buffer_size: 1024 * 1024,  // 1MB
    chan_capacity: 1000,       // Taille du canal
    timeout: 1000,             // 1s
    snaplen: 65535,            // Taille max des paquets
};

// Démarrage de la capture
let handle = CaptureHandle::new();
handle.start(config, app_handle, event_channel)?;

// ... utilisation ...

// Arrêt de la capture
handle.stop(event_channel)?;
```

## Dépannage

### Problèmes courants

1. **Permissions insuffisantes**
   - Vérifier les droits d'accès à l'interface réseau
   - Exécuter avec les privilèges nécessaires

2. **Perte de paquets**
   - Augmenter la taille du buffer
   - Vérifier la charge CPU
   - Ajuster les filtres BPF

3. **Latence élevée**
   - Réduire la taille du canal
   - Optimiser le traitement des paquets

## Sécurité

- Les paquets sont traités de manière sécurisée
- Les données sensibles ne sont pas conservées en mémoire
- Les filtres BPF sont validés avant utilisation

## Limitations

- Dépendance à la bibliothèque `pcap`
- Performances limitées par le matériel réseau
- Nécessite des privilèges élevés pour la capture
