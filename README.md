# embassy-gy-bmi160 (v0.2.0)  
Driver async no_std pour l'unité de mesure inertielle (IMU) 6 axes BMI160.

Conçu spécifiquement pour l'écosystème Embassy (embassy-time, embassy-sync) . Ce driver se concentre sur l'extraction  des données brutes via I2C.

# Update La version 0.2.0 , le vip : #![forbid(unsafe_code)]
integre un exemple OLEd sqrt gyro-acce, clé en main et du safety avec #![forbid(unsafe_code)]

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
embassy-gy-bmi160 = "0.2.0"
embassy-time      = { version = "0.4.0" }
embassy-sync      = { version = "0.6.0" }
embedded-hal-async = { version = "1.0" }
# Exemple Complet : Intégration JC-OS (LCD + IMU) plus embedded-f32-sqrt pour laffichage 
Voici comment orchestrer la lecture du capteur et l'affichage sur un LCD en utilisant les signaux globaux, vous pouvez rétrouver lensemble des crates dans mon profil , des crates plug and play :) .

````rust

#![no_std]
#![no_main]


use cortex_m_rt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{Config as I2cConfig, I2c, Async};
use embassy_time::{Delay, Duration, Timer, with_timeout};
//ma crate pour afficher asynchro :)
use hd44780_i2c_nostd::LcdI2c;
use {panic_halt as _, embassy_rp as _};
use heapless::String;
use core::fmt::Write;

// Partage de bus I2C
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

// Utilisation de LA nouvelle crate 
use embassy_gy_bmi160::Bmi160;
use embassy_gy_bmi160::signals::{ACCEL_SIGNAL, GYRO_SIGNAL};

// Mathématiques pour l'affichage :)
use embedded_f32_sqrt::sqrt;

use rp2350_linker as _;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::I2C0; 

bind_interrupts!(struct Irqs {
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<I2C0>;
});

static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C0, Async>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(embassy_rp::config::Config::default());
    
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 100_000;
    
    // Initialisation du bus partagé
    let i2c = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, i2c_config);
    let i2c_bus = Mutex::<NoopRawMutex, _>::new(i2c);
    let i2c_bus = I2C_BUS.init(i2c_bus);

    let lcd_i2c = I2cDevice::new(i2c_bus);
    let bmi_i2c = I2cDevice::new(i2c_bus);

    let lcd = LcdI2c::new(lcd_i2c, 0x3F); 
    let bmi = Bmi160::new(bmi_i2c, 0x68);

    // Lancement de la tâche système
    spawner.spawn(system_task(lcd, bmi)).unwrap();

    // LED de statut (Heartbeat)
    let mut led = Output::new(p.PIN_25, Level::Low);
    loop {
        led.toggle();
        Timer::after_millis(200).await; 
    }
}

#[embassy_executor::task]
async fn system_task(
    mut lcd: LcdI2c<I2cDevice<'static, NoopRawMutex, I2c<'static, I2C0, Async>>>,
    mut bmi: Bmi160<'static, I2cDevice<'static, NoopRawMutex, I2c<'static, I2C0, Async>>>
) {
    let mut delay = Delay;
    let mut imu_ready = false;
    
    // 1. Splash Screen
    if lcd.init(&mut delay).await.is_ok() {
        let _ = lcd.set_backlight(true);
        let _ = lcd.clear(&mut delay).await;
        let _ = lcd.write_str("THE RUST EAGLE", &mut delay).await;
    }

    // 2. Initialisation IMU via ton Driver
    if let Ok(Ok(_)) = with_timeout(Duration::from_millis(150), bmi.init()).await {
        imu_ready = true;
    } else {
        // Tentative sur adresse alternative si SA0 est au 3.3V
        bmi.set_address(0x69);
        if let Ok(Ok(_)) = with_timeout(Duration::from_millis(150), bmi.init()).await {
            imu_ready = true;
        }
    }

    loop {
        if imu_ready {
            // Lecture Gyro -> Publication via Signal
            if let Ok(g) = bmi.read_gyro().await {
                GYRO_SIGNAL.signal(g);
                
                let _ = lcd.set_cursor(0, 0, &mut delay).await;
                let mut s: String<16> = String::new();
                let _ = write!(s, "G:{:>4} {:>4} {:>4}", g.x/128, g.y/128, g.z/128);
                let _ = lcd.write_str(s.as_str(), &mut delay).await;
            }

            // Lecture Accel -> Publication via Signal
            if let Ok(a) = bmi.read_accel().await {
                ACCEL_SIGNAL.signal(a);

                let x = a.x as f32; 
                let y = a.y as f32; 
                let z = a.z as f32;

                //utilisation de la crate plug and play embedded-f32-sqrt 
                let mag = sqrt(x*x + y*y + z*z).unwrap_or(0.0) / 16384.0;

                let _ = lcd.set_cursor(1, 0, &mut delay).await;
                let mut s: String<16> = String::new();
                let _ = write!(s, "ACCEL: {:.3}G 🦅", mag);
                let _ = lcd.write_str(s.as_str(), &mut delay).await;
            }
        } else {
            let _ = lcd.set_cursor(1, 0, &mut delay).await;
            let _ = lcd.write_str("IMU NOT FOUND", &mut delay).await;
        }
        
        Timer::after_millis(150).await;
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

Pour le peuple , les makers , un grand merci sans vous tout ca serait pas possible , Merci Rust , Merci Raspberry pi.

# Licence
Ce projet est distribué sous licence GPL-2.0-or-later.
Voir le fichier LICENSE pour les détails.


Copyright (C) 2026 Jorge Andre Castro