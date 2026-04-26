# Changelog

Tous les changements notables de ce projet sont documentés dans ce fichier.

Le format suit [Keep a Changelog](https://keepachangelog.com/), et le versioning suit [Semantic Versioning](https://semver.org/).

## [0.2.1] - 2026-04-26

### Changed
- **Expanded embassy-time support** : Élargissement de la compatibilité avec les versions d'embassy-time (`>=0.3, <0.6`) pour une flexibilité accrue sur les versions de dépendances
- **Expanded embassy-sync support** : Support étendu d'embassy-sync (`>=0.4, <0.9`) permettant l'utilisation avec plusieurs versions de l'écosystème Embassy
- Amélioration de la compatibilité avec l'écosystème Embassy pour supporter un plus large éventail de projets
- Ajout d'un exemple minimal pour utiliser les fonctions de la crate , set_address , read_gyro , read_accel

## [0.2.0] - 2026-02-15

### Added
- **Forbid unsafe code** : Intégration de `#![forbid(unsafe_code)]` pour garantir une sécurité maximale
- **Exemple OLED complet** : Ajout d'un exemple clé en main avec affichage OLED, calcul sqrt gyro-accéléromètre
- **Découplage par signaux** : Module `signals` intégré pour la communication inter-tâches fluide

### Changed
- Refactorisation pour zéro allocation et bare-metal pur

## [0.1.0] - 2025-12-01

### Added
- Driver async initial pour le capteur BMI160
- Support de la lecture I2C non-bloquante via `embedded-hal-async`
- Architecture `no_std` compatible avec Embassy
- Données brutes i16 sans logique superflue
- Support du bus I2C partagé
