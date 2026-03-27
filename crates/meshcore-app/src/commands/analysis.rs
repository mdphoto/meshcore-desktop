use meshcore_service::AppState;
use meshcore_service::los::{self, LosResult};
use tauri::State;

/// Analyse la ligne de vue entre deux points
/// Retourne le profil d'élévation, la visibilité et les données Fresnel
#[tauri::command]
pub fn analyze_los(
    state: State<'_, AppState>,
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
    freq_mhz: f64,
    ant_height1: f64,
    ant_height2: f64,
) -> Result<LosResult, String> {
    let result = los::analyze_los(
        &state.srtm_cache,
        lat1, lon1, ant_height1,
        lat2, lon2, ant_height2,
        freq_mhz,
        200, // 200 points d'échantillonnage
    );
    Ok(result)
}

/// Récupère l'altitude d'un point (pour affichage dans les popups)
#[tauri::command]
pub fn get_elevation(
    state: State<'_, AppState>,
    lat: f64,
    lon: f64,
) -> Result<Option<f64>, String> {
    Ok(state.srtm_cache.get_elevation(lat, lon))
}
