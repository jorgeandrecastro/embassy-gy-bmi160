// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Signaux globaux asynchrones pour le partage de données IMU.
//! 
//! Utilisent des mécanismes de synchronisation `embassy-sync` 
//! compatibles avec les interruptions (CriticalSection).
#![forbid(unsafe_code)]
use crate::BmiData;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

/// Canal de diffusion pour la dernière mesure de l'accéléromètre.
pub static ACCEL_SIGNAL: Signal<CriticalSectionRawMutex, BmiData> = Signal::new();

/// Canal de diffusion pour la dernière mesure du gyroscope.
pub static GYRO_SIGNAL: Signal<CriticalSectionRawMutex, BmiData> = Signal::new();