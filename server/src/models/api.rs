use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct MapBounds {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteGeneration {
    pub name: String,
    pub radius: f64,
    pub generations: usize,
}