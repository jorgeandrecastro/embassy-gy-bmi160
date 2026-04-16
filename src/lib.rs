// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

#![no_std]
#![forbid(unsafe_code)]

//! # embassy-gy-bmi160
//!
//! Driver asynchrone `no_std` pour l'IMU Bosch BMI160 via I2C.
//! Développé pour le Kernel JC-OS, optimisé pour l'exécuteur `embassy`.
//! 
//! Ce pilote fournit un accès direct aux registres de données brutes 
//! de l'accéléromètre et du gyroscope sans surcoût de calcul.

pub mod signals;

use core::marker::PhantomData;
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;

/// Données de mouvement brutes sur trois axes (X, Y, Z).
/// Représentées par des entiers signés de 16 bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BmiData {
    /// Valeur brute sur l'axe X.
    pub x: i16,
    /// Valeur brute sur l'axe Y.
    pub y: i16,
    /// Valeur brute sur l'axe Z.
    pub z: i16,
}

/// Instance principale du driver BMI160.
/// 
/// Fonctionne avec n'importe quel périphérique implémentant `embedded-hal-async::i2c::I2c`.
pub struct Bmi160<'d, I>
where
    I: I2c,
{
    i2c: I,
    /// Adresse I2C configurée (0x68 ou 0x69).
    pub addr: u8,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, I> Bmi160<'d, I>
where
    I: I2c,
{
    /// Initialise une nouvelle instance du driver.
    /// 
    /// # Arguments
    /// * `i2c` - Bus I2C (ou I2cDevice partagé).
    /// * `addr` - Adresse du composant (généralement 0x68).
    pub fn new(i2c: I, addr: u8) -> Self {
        Self {
            i2c,
            addr,
            _phantom: PhantomData,
        }
    }

    /// Met à jour l'adresse I2C dynamiquement.
    pub fn set_address(&mut self, new_addr: u8) {
        self.addr = new_addr;
    }

    /// Configure le capteur en "Normal Mode" pour l'accéléromètre et le gyroscope.
    /// 
    /// Envoie les commandes PMU aux registres de contrôle d'alimentation.
    pub async fn init(&mut self) -> Result<(), I::Error> {
        // Accéléromètre en mode Normal (Commande 0x11)
        self.i2c.write(self.addr, &[0x7E, 0x11]).await?;
        Timer::after_millis(10).await;

        // Gyroscope en mode Normal (Commande 0x15)
        self.i2c.write(self.addr, &[0x7E, 0x15]).await?;
        Timer::after_millis(10).await;

        Ok(())
    }

    /// Lit les données brutes du gyroscope (registres 0x0C à 0x11).
    pub async fn read_gyro(&mut self) -> Result<BmiData, I::Error> {
        let mut data = [0u8; 6];
        self.i2c.write_read(self.addr, &[0x0C], &mut data).await?;
        Ok(BmiData {
            x: i16::from_le_bytes([data[0], data[1]]),
            y: i16::from_le_bytes([data[2], data[3]]),
            z: i16::from_le_bytes([data[4], data[5]]),
        })
    }

    /// Lit les données brutes de l'accéléromètre (registres 0x12 à 0x17).
    pub async fn read_accel(&mut self) -> Result<BmiData, I::Error> {
        let mut data = [0u8; 6];
        self.i2c.write_read(self.addr, &[0x12], &mut data).await?;
        Ok(BmiData {
            x: i16::from_le_bytes([data[0], data[1]]),
            y: i16::from_le_bytes([data[2], data[3]]),
            z: i16::from_le_bytes([data[4], data[5]]),
        })
    }
}