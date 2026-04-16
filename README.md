# embassy-gy-bmi160 (v0.1.0) 
Driver async no_std pour l'unité de mesure inertielle (IMU) 6 axes BMI160.

Conçu spécifiquement pour l'écosystème Embassy (embassy-time, embassy-sync) . Ce driver se concentre sur l'extraction  des données brutes via I2C.

# 🔴 ATTENTION MATÉRIEL
Le module GY-BMI160 (HW-661) nécessite une attention particulière pour fonctionner sur un bus I2C partagé :

CS (Chip Select) : Doit être relié au 3.3V pour activer le mode I2C.

SA0 : Définit l'adresse. Reliez au GND pour l'adresse 0x68 (par défaut dans ce driver).

Fonctionnalités
Async Natif : Utilise embedded-hal-async pour des lectures I2C non-bloquantes.

Zéro Logique Superflue : Retourne uniquement les données brutes (i16) pour une latence minimale.

Bus Partagé : Compatible avec embassy-embedded-hal pour cohabiter avec d'autres périphériques (LCD, etc.).

Découplage par Signaux : Module signals intégré pour une communication inter-tâches fluide.

Zéro Allocation : Conçu pour le bare-metal pur.

# Installation
Ini, TOML
[dependencies]
embassy-gy-bmi160 = "0.1.0"
embassy-time      = { version = "0.4.0" }
embassy-sync      = { version = "0.6.0" }
embedded-hal-async = { version = "1.0" }
Exemple Complet : Intégration JC-OS (LCD + IMU)
Voici comment orchestrer la lecture du capteur et l'affichage sur un LCD en utilisant les signaux globaux.

````rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, Delay};
use embassy_gy_bmi160::Bmi160;
use embassy_gy_bmi160::signals::{ACCEL_SIGNAL, GYRO_SIGNAL};
use core::fmt::Write;
use heapless::String;

#[embassy_executor::task]
async fn imu_task(mut bmi: Bmi160<'static, MyI2cDevice>) {
    // Initialisation (Accel + Gyro en mode Normal)
    if bmi.init().await.is_ok() {
        loop {
            // Lecture des données brutes
            if let Ok(accel) = bmi.read_accel().await {
                ACCEL_SIGNAL.signal(accel);
            }
            if let Ok(gyro) = bmi.read_gyro().await {
                GYRO_SIGNAL.signal(gyro);
            }
            Timer::after(Duration::from_millis(100)).await;
        }
    }
}

#[embassy_executor::task]
async fn display_task(mut lcd: MyLcd) {
    let mut delay = Delay;
    loop {
        // Attend la mise à jour des signaux
        let a = ACCEL_SIGNAL.wait().await;
        let g = GYRO_SIGNAL.wait().await;

        let mut s: String<16> = String::new();
        let _ = write!(s, "G:{:>4} {:>4} {:>4}", g.x/128, g.y/128, g.z/128);
        
        let _ = lcd.set_cursor(0, 0, &mut delay).await;
        let _ = lcd.write_str(s.as_str(), &mut delay).await;
        
        // Affichage de l'accélération sur la ligne 2...
    }
}
````
# Pourquoi cette architecture ?
L'utilisation des signaux ACCEL_SIGNAL et GYRO_SIGNAL garantit la stabilité du Kernel :

Priorité Critique : La tâche IMU peut tourner à haute fréquence sans être ralentie par la lenteur de l'affichage LCD.

Consommation Prédictive : Les tâches consommatrices attendent (wait()) passivement, libérant les cycles CPU du RP2350.

Sécurité : Aucun partage de mémoire complexe, tout passe par des primitives de synchronisation no_std.

# Schéma de Câblage (Pico 2)
Respectez ce montage pour garantir l'intégrité du bus I2C :

VCC : 3.3V

GND : Masse commune

SCL / SDA : Pins GP5 / GP4 (avec pull-ups si nécessaire)

CS : IMPÉRATIF au 3.3V (Active le mode I2C)

SA0 : Au GND (Fixe l'adresse à 0x68)


Copyright (C) 2026 Jorge Andre Castro



# Licence
Ce projet est distribué sous licence GPL-2.0-or-later.
Voir le fichier LICENSE pour les détails.