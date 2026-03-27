//! Reconnexion automatique avec backoff exponentiel

use crate::TransportError;
use crate::manager::{ConnectionManager, ConnectionTarget};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Configuration de la stratégie de reconnexion
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Délai initial entre les tentatives
    pub initial_delay: Duration,
    /// Délai maximum entre les tentatives
    pub max_delay: Duration,
    /// Multiplicateur du délai à chaque tentative
    pub multiplier: f64,
    /// Nombre maximum de tentatives (0 = infini)
    pub max_retries: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_retries: 0, // infini
        }
    }
}

/// Tente de reconnecter avec backoff exponentiel
pub async fn reconnect_with_backoff(
    manager: &mut ConnectionManager,
    target: &ConnectionTarget,
    config: &ReconnectConfig,
) -> Result<(), TransportError> {
    let mut delay = config.initial_delay;
    let mut attempts = 0u32;

    loop {
        attempts += 1;
        info!("Tentative de reconnexion #{}", attempts);

        match manager.connect(target.clone()).await {
            Ok(()) => {
                info!("Reconnexion réussie après {} tentative(s)", attempts);
                return Ok(());
            }
            Err(e) => {
                warn!("Échec tentative #{} : {}", attempts, e);

                if config.max_retries > 0 && attempts >= config.max_retries {
                    return Err(TransportError::Connection(format!(
                        "Échec après {} tentatives : {}",
                        attempts, e
                    )));
                }

                sleep(delay).await;
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.multiplier).min(config.max_delay.as_secs_f64()),
                );
            }
        }
    }
}
