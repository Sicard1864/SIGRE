use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Compteur à fenêtre fixe, globale par route, en mémoire.
///
/// Volontairement simple (choix de la tâche 4 : « au plus simple ») :
/// suffisant pour un seul process back en V1 ; le jour du multi-instances,
/// remplacer par un stockage partagé (ex. Redis). Le STYLE_API.md § 8 impose
/// le `429` + `Retry-After` sur login, refresh et inscription.
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<&'static str, Bucket>>>,
}

struct Bucket {
    window_started_at: Instant,
    count: u32,
}

/// Une règle nommée : clé de compteur, plafond, fenêtre.
#[derive(Clone, Copy)]
pub struct Rule {
    pub key: &'static str,
    pub max_requests: u32,
    pub window: Duration,
}

pub const LOGIN: Rule = Rule {
    key: "auth.login",
    max_requests: 10,
    window: Duration::from_secs(60),
};

pub const REFRESH: Rule = Rule {
    key: "auth.refresh",
    max_requests: 10,
    window: Duration::from_secs(60),
};

pub const REGISTER: Rule = Rule {
    key: "accounts.register",
    max_requests: 5,
    window: Duration::from_secs(60),
};

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Consomme un jeton de la règle. `Err(secondes)` = limite atteinte,
    /// le délai est exposé dans `Retry-After`.
    pub fn check(&self, rule: Rule) -> Result<(), u32> {
        // Un Mutex empoisonné signifie qu'un panic a eu lieu ailleurs :
        // la HashMap reste structurellement valide, on la récupère.
        let mut buckets = self
            .buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = Instant::now();

        match buckets.get_mut(rule.key) {
            Some(bucket) if now.duration_since(bucket.window_started_at) < rule.window => {
                if bucket.count >= rule.max_requests {
                    let elapsed = now.duration_since(bucket.window_started_at);
                    // u64 vers u32 : la fenêtre fait 60 s, le cast est large.
                    Err((rule.window - elapsed).as_secs().max(1) as u32)
                } else {
                    bucket.count += 1;
                    Ok(())
                }
            }
            // Fenêtre expirée ou premier appel : on repart d'une fenêtre neuve.
            _ => {
                buckets.insert(rule.key, Bucket { window_started_at: now, count: 1 });
                Ok(())
            }
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
