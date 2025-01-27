use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;
use std::vec;

use geo::{Coord, HaversineDistance, Point};
use geohash::encode;

use crate::utils;
use model::api::{point_array::PointArray, single_vec::SingleVec};

pub fn multi(clusters: &SingleVec, route_split_level: usize) -> SingleVec {
    let time = Instant::now();

    let get_hash = |point: PointArray, modifier: usize| {
        encode(
            Coord {
                x: point[1],
                y: point[0],
            },
            route_split_level + modifier,
        )
        .unwrap()
    };

    let mut point_map: HashMap<String, SingleVec> = HashMap::new();
    for point in clusters.into_iter() {
        let key = get_hash(*point, 0);
        point_map
            .entry(key)
            .and_modify(|x| x.push(*point))
            .or_insert(vec![*point]);
    }

    let merged_routes: Vec<(PointArray, SingleVec)> = point_map
        .iter()
        .enumerate()
        .map(|(i, (hash, segment))| {
            log::debug!("Creating thread: {} for hash {}", i + 1, hash);
            let mut route = or_tools(&segment);
            if let Some(last) = route.last() {
                if let Some(first) = route.first() {
                    if first == last {
                        route.pop();
                    }
                }
            }
            (
                if route.len() > 0 {
                    utils::centroid(&route)
                } else {
                    [0., 0.]
                },
                route,
            )
        })
        .collect();
    let mut centroids = vec![];

    point_map.clear();
    merged_routes
        .into_iter()
        .enumerate()
        .for_each(|(_i, (hash, r))| {
            centroids.push(hash);
            point_map.insert(get_hash(hash, 0), r);
        });

    let clusters: Vec<SingleVec> = or_tools(&centroids)
        .into_iter()
        .filter_map(|c| {
            let hash = get_hash(c, 0);
            point_map.remove(&hash)
        })
        .collect();

    let mut final_routes: SingleVec = vec![];

    for (i, current) in clusters.clone().iter_mut().enumerate() {
        let next: &SingleVec = if i == clusters.len() - 1 {
            clusters[0].as_ref()
        } else {
            clusters[i + 1].as_ref()
        };

        let mut shortest = std::f64::MAX;
        let mut shortest_current_index = 0;

        for (current_index, current_point) in current.iter().enumerate() {
            let current_point = Point::new(current_point[1], current_point[0]);
            for (_next_index, next_point) in next.iter().enumerate() {
                let next_point = Point::new(next_point[1], next_point[0]);
                let distance = current_point.haversine_distance(&next_point);
                if distance < shortest {
                    shortest = distance;
                    shortest_current_index = current_index;
                }
            }
        }
        current.rotate_left(shortest_current_index);
        final_routes.append(current);
    }

    log::info!("[TSP] time: {}", time.elapsed().as_secs_f32());
    final_routes
}

fn directory() -> std::io::Result<String> {
    let mut path = std::env::current_dir()?;
    path.push("algorithms");
    path.push("src");
    path.push("routing");
    path.push("tsp");
    if path.exists() {
        Ok(path.display().to_string())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "TSP solver does not exist, rerun the OR Tools Script",
        ))
    }
}

pub fn stringify_points(points: &SingleVec) -> String {
    points
        .iter()
        .enumerate()
        .map(|(i, cluster)| {
            format!(
                "{},{}, {}",
                cluster[0],
                cluster[1],
                if i == points.len() - 1 { "" } else { "," }
            )
        })
        .collect::<String>()
}

pub fn or_tools(clusters: &SingleVec) -> SingleVec {
    let time = Instant::now();
    log::debug!("[TSP] Starting");
    let mut result = vec![];

    let stringified_points = stringify_points(clusters);

    if let Ok(dir) = directory() {
        let mut child = match Command::new(&dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => {
                log::error!("[TSP] to spawn child process {}", err);
                return vec![];
            }
        };

        let mut stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                log::error!("[TSP] Failed to open stdin");
                return vec![];
            }
        };

        std::thread::spawn(
            move || match stdin.write_all(stringified_points.as_bytes()) {
                Ok(_) => match stdin.flush() {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("[TSP] Failed to flush stdin: {}", err);
                    }
                },
                Err(err) => {
                    log::error!("[TSP] Failed to write to stdin: {}", err)
                }
            },
        );

        let output = match child.wait_with_output() {
            Ok(result) => result,
            Err(err) => {
                log::error!("[TSP] Failed to read stdout: {}", err);
                return vec![];
            }
        };
        let output = String::from_utf8_lossy(&output.stdout);
        let output = output
            .split(",")
            .filter_map(|s| s.parse::<usize>().ok())
            .collect::<Vec<usize>>();

        output.into_iter().for_each(|i| {
            result.push(clusters[i]);
        });
    } else {
        log::error!("[TSP] solver not found, rerun the OR-Tools script to generate it");
    }
    log::debug!("[TSP] Finished in {}s", time.elapsed().as_secs_f32());
    result
}
