//! # Exemple Simple : Lecture du Capteur BMI160
//! 
//! Cet exemple montre comment :
//! 1. Initialiser le capteur BMI160
//! 2. Lire les données du gyroscope
//! 3. Lire les données de l'accéléromètre

#![no_std]
#![no_main]

use cortex_m_rt as _;
use embassy_executor::Spawner;
use embassy_rp::i2c::{Config as I2cConfig, I2c, Async};
use embassy_time::Timer;
use {panic_halt as _, embassy_rp as _};

use static_cell::StaticCell;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

use embassy_gy_bmi160::Bmi160;

use rp2350_linker as _;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::I2C0;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<I2C0>;
});

static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C0, Async>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(embassy_rp::config::Config::default());

    // === Initialisation du Bus I2C ===
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 100_000;
    
    let i2c = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, i2c_config);
    let i2c_bus = Mutex::<NoopRawMutex, _>::new(i2c);
    let i2c_bus = I2C_BUS.init(i2c_bus);

    // === Création du Device BMI160 ===
    let bmi_i2c = I2cDevice::new(i2c_bus);
    let mut bmi160 = Bmi160::new(bmi_i2c, 0x68);

    // === Initialisation du Capteur ===
    if bmi160.init().await.is_err() {
        loop {
            Timer::after_millis(100).await;
        }
    }

    // === Boucle de Lecture ===
    loop {
        // Lire l'accéléromètre
        if let Ok(accel) = bmi160.read_accel().await {
            let _ = accel; // Données disponibles pour utilisation
        }

        // Lire le gyroscope
        if let Ok(gyro) = bmi160.read_gyro().await {
            let _ = gyro; // Données disponibles pour utilisation
        }

        Timer::after_millis(100).await;
    }
}
