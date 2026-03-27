//! Analyse de ligne de vue (Line of Sight) avec données d'élévation SRTM
//!
//! Télécharge les tuiles SRTM (Shuttle Radar Topography Mission) à la demande,
//! les cache localement, et calcule la visibilité entre deux points en tenant
//! compte du relief et de la zone de Fresnel.

use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{info, warn};

/// Résolution SRTM : 1 arc-seconde (~30m), 3601x3601 points par tuile
const SRTM_SIZE: usize = 3601;
/// Rayon de la Terre en mètres
const EARTH_RADIUS: f64 = 6_371_000.0;

/// Résultat d'analyse ligne de vue
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LosResult {
    /// true si la ligne de vue est dégagée
    pub visible: bool,
    /// Distance totale en mètres
    pub distance_m: f64,
    /// Profil d'élévation (distance_m, elevation_m, terrain_m, fresnel_clearance_m)
    pub profile: Vec<ProfilePoint>,
    /// Point d'obstruction le plus critique (si obstrué)
    pub worst_obstruction: Option<ObstructionPoint>,
    /// Clearance minimale de la zone de Fresnel (négatif = obstrué)
    pub min_fresnel_clearance_m: f64,
}

/// Point du profil d'élévation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfilePoint {
    /// Distance depuis le point A en mètres
    pub distance_m: f64,
    /// Altitude du terrain en mètres (SRTM)
    pub terrain_m: f64,
    /// Altitude de la ligne de vue directe (interpolation linéaire A→B)
    pub los_m: f64,
    /// Rayon de la 1ère zone de Fresnel à ce point
    pub fresnel_radius_m: f64,
    /// Latitude du point
    pub lat: f64,
    /// Longitude du point
    pub lon: f64,
}

/// Point d'obstruction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObstructionPoint {
    pub distance_m: f64,
    pub terrain_m: f64,
    pub los_m: f64,
    pub clearance_m: f64,
    pub lat: f64,
    pub lon: f64,
}

/// Cache de tuiles SRTM en mémoire
pub struct SrtmCache {
    cache_dir: PathBuf,
    tiles: Mutex<HashMap<(i16, i16), Vec<i16>>>,
}

impl SrtmCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&cache_dir);
        Self {
            cache_dir,
            tiles: Mutex::new(HashMap::new()),
        }
    }

    /// Récupère l'altitude d'un point (lat, lon) en mètres
    pub fn get_elevation(&self, lat: f64, lon: f64) -> Option<f64> {
        let tile_lat = lat.floor() as i16;
        let tile_lon = lon.floor() as i16;

        let mut tiles = self.tiles.lock().ok()?;

        if !tiles.contains_key(&(tile_lat, tile_lon)) {
            if let Some(data) = self.load_tile(tile_lat, tile_lon) {
                tiles.insert((tile_lat, tile_lon), data);
            } else {
                return None;
            }
        }

        let tile = tiles.get(&(tile_lat, tile_lon))?;

        // Position dans la tuile (row=0 est le nord du degré)
        let row = ((tile_lat as f64 + 1.0 - lat) * (SRTM_SIZE - 1) as f64) as usize;
        let col = ((lon - tile_lon as f64) * (SRTM_SIZE - 1) as f64) as usize;

        if row >= SRTM_SIZE || col >= SRTM_SIZE {
            return None;
        }

        let idx = row * SRTM_SIZE + col;
        let elev = tile.get(idx).copied()?;

        // -32768 = void (pas de données)
        if elev == -32768 {
            None
        } else {
            Some(elev as f64)
        }
    }

    /// Charge une tuile SRTM depuis le cache disque ou la télécharge
    fn load_tile(&self, lat: i16, lon: i16) -> Option<Vec<i16>> {
        let filename = srtm_filename(lat, lon);
        let hgt_path = self.cache_dir.join(&filename);

        // Essayer depuis le disque
        if hgt_path.exists() {
            return self.parse_hgt_file(&hgt_path);
        }

        // Télécharger depuis le serveur SRTM public
        info!("Téléchargement tuile SRTM : {}", filename);
        match self.download_tile(lat, lon) {
            Ok(data) => {
                // Sauvegarder sur disque
                if let Err(e) = fs::write(&hgt_path, &data) {
                    warn!("Impossible de cacher la tuile SRTM : {}", e);
                }
                self.parse_hgt_bytes(&data)
            }
            Err(e) => {
                warn!("Échec téléchargement SRTM {} : {}", filename, e);
                None
            }
        }
    }

    fn download_tile(&self, lat: i16, lon: i16) -> Result<Vec<u8>, String> {
        let filename = srtm_filename(lat, lon);
        // Source : serveur SRTM NASA via USGS (données publiques, pas d'auth requise pour SRTM1)
        // Alternative : viewfinderpanoramas.org qui héberge les données librement
        let url = format!(
            "https://elevation-tiles-prod.s3.amazonaws.com/skadi/{}/{}.hgt.gz",
            srtm_dir_name(lat, lon),
            filename.trim_end_matches(".hgt")
        );

        let resp = ureq::get(&url)
            .call()
            .map_err(|e| format!("HTTP error: {}", e))?;

        let compressed = resp
            .into_body()
            .read_to_vec()
            .map_err(|e| format!("Read error: {}", e))?;

        // Décompresser gzip
        let mut decoder = flate2::read::GzDecoder::new(Cursor::new(compressed));
        let mut data = Vec::new();
        decoder
            .read_to_end(&mut data)
            .map_err(|e| format!("Décompression error: {}", e))?;

        Ok(data)
    }

    fn parse_hgt_file(&self, path: &Path) -> Option<Vec<i16>> {
        let data = fs::read(path).ok()?;
        self.parse_hgt_bytes(&data)
    }

    fn parse_hgt_bytes(&self, data: &[u8]) -> Option<Vec<i16>> {
        let expected = SRTM_SIZE * SRTM_SIZE * 2;
        if data.len() != expected {
            warn!(
                "Taille tuile SRTM invalide : {} (attendu {})",
                data.len(),
                expected
            );
            return None;
        }

        let mut elevations = Vec::with_capacity(SRTM_SIZE * SRTM_SIZE);
        for i in (0..data.len()).step_by(2) {
            // Big-endian signed 16-bit
            let value = i16::from_be_bytes([data[i], data[i + 1]]);
            elevations.push(value);
        }
        Some(elevations)
    }
}

/// Calcule la ligne de vue entre deux points
pub fn analyze_los(
    cache: &SrtmCache,
    lat1: f64,
    lon1: f64,
    height1: f64,
    lat2: f64,
    lon2: f64,
    height2: f64,
    freq_mhz: f64,
    num_samples: usize,
) -> LosResult {
    let distance = haversine(lat1, lon1, lat2, lon2);
    let mut profile = Vec::with_capacity(num_samples);
    let mut min_clearance = f64::MAX;
    let mut worst_obstruction: Option<ObstructionPoint> = None;

    for i in 0..num_samples {
        let t = i as f64 / (num_samples - 1) as f64;

        // Interpolation linéaire des coordonnées
        let lat = lat1 + (lat2 - lat1) * t;
        let lon = lon1 + (lon2 - lon1) * t;
        let d = distance * t;

        // Altitude du terrain
        let terrain = cache.get_elevation(lat, lon).unwrap_or(0.0);

        // Ligne de vue directe (avec courbure de la Terre)
        let earth_curvature = earth_curvature_correction(d, distance);
        let los_height = height1
            + (height2 - height1) * t
            + cache.get_elevation(lat1, lon1).unwrap_or(0.0) * (1.0 - t)
            + cache.get_elevation(lat2, lon2).unwrap_or(0.0) * t
            - earth_curvature;

        // Zone de Fresnel (1ère zone)
        let fresnel_r = if freq_mhz > 0.0 && d > 0.0 && (distance - d) > 0.0 {
            fresnel_radius(freq_mhz, d, distance - d)
        } else {
            0.0
        };

        let clearance = los_height - terrain - fresnel_r;
        if clearance < min_clearance {
            min_clearance = clearance;
            if clearance < 0.0 {
                worst_obstruction = Some(ObstructionPoint {
                    distance_m: d,
                    terrain_m: terrain,
                    los_m: los_height,
                    clearance_m: clearance,
                    lat,
                    lon,
                });
            }
        }

        profile.push(ProfilePoint {
            distance_m: d,
            terrain_m: terrain,
            los_m: los_height,
            fresnel_radius_m: fresnel_r,
            lat,
            lon,
        });
    }

    LosResult {
        visible: min_clearance >= 0.0,
        distance_m: distance,
        profile,
        worst_obstruction,
        min_fresnel_clearance_m: min_clearance,
    }
}

/// Distance haversine entre deux points (mètres)
fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    EARTH_RADIUS * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

/// Correction de courbure terrestre (m) pour un trajet de distance totale `total_d`
/// au point situé à `d` du départ. Utilise le facteur k=4/3 (réfraction standard).
fn earth_curvature_correction(d: f64, total_d: f64) -> f64 {
    let k = 4.0 / 3.0; // facteur de réfraction atmosphérique
    let d2 = total_d - d;
    (d * d2) / (2.0 * k * EARTH_RADIUS)
}

/// Rayon de la 1ère zone de Fresnel (mètres)
/// freq_mhz : fréquence en MHz
/// d1, d2 : distances depuis chaque extrémité en mètres
fn fresnel_radius(freq_mhz: f64, d1: f64, d2: f64) -> f64 {
    let wavelength = 300.0 / freq_mhz; // longueur d'onde en mètres
    let total = d1 + d2;
    if total <= 0.0 {
        return 0.0;
    }
    (wavelength * d1 * d2 / total).sqrt()
}

/// Nom du fichier SRTM pour une tuile
fn srtm_filename(lat: i16, lon: i16) -> String {
    let ns = if lat >= 0 { 'N' } else { 'S' };
    let ew = if lon >= 0 { 'E' } else { 'W' };
    format!(
        "{}{:02}{}{:03}.hgt",
        ns,
        lat.unsigned_abs(),
        ew,
        lon.unsigned_abs()
    )
}

/// Nom du répertoire SRTM (ex: "N44" pour la latitude 44)
fn srtm_dir_name(lat: i16, _lon: i16) -> String {
    let ns = if lat >= 0 { 'N' } else { 'S' };
    format!("{}{:02}", ns, lat.unsigned_abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srtm_filename() {
        assert_eq!(srtm_filename(44, 4), "N44E004.hgt");
        assert_eq!(srtm_filename(-12, -70), "S12W070.hgt");
        assert_eq!(srtm_filename(0, 0), "N00E000.hgt");
    }

    #[test]
    fn test_haversine() {
        // Paris → Lyon ≈ 392 km
        let d = haversine(48.8566, 2.3522, 45.7578, 4.8320);
        assert!((d - 392_000.0).abs() < 5000.0);
    }

    #[test]
    fn test_fresnel_radius() {
        // À 915 MHz, point milieu d'un lien de 10 km
        // λ = 300/915 ≈ 0.328m, F1 = sqrt(λ*d1*d2/D) = sqrt(0.328*5000*5000/10000) ≈ 28.6m
        let r = fresnel_radius(915.0, 5000.0, 5000.0);
        assert!(r > 20.0 && r < 40.0, "Fresnel radius: {} m", r);
    }

    #[test]
    fn test_earth_curvature() {
        // À 10 km de distance, la courbure au milieu devrait être ~6-10m
        let c = earth_curvature_correction(5000.0, 10000.0);
        assert!(c > 1.0 && c < 15.0, "Curvature: {} m", c);
    }
}
