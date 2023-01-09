use super::{multi_vec::MultiVec, poracle::PoracleId, *};

pub trait Default {
    fn default() -> Self;
}

impl Default for FeatureCollection {
    fn default() -> FeatureCollection {
        FeatureCollection {
            bbox: None,
            foreign_members: None,
            features: vec![],
        }
    }
}

impl ToSingleVec for FeatureCollection {
    fn to_single_vec(self) -> single_vec::SingleVec {
        self.to_multi_vec().into_iter().flatten().collect()
    }
}

impl EnsurePoints for FeatureCollection {
    fn ensure_first_last(self) -> Self {
        self.into_iter()
            .map(|feat| feat.ensure_first_last())
            .collect()
    }
}

impl GeometryHelpers for FeatureCollection {
    fn simplify(self) -> Self {
        self.into_iter()
            .map(|feat| {
                if let Some(geometry) = feat.geometry {
                    Feature {
                        geometry: Some(geometry.simplify()),
                        ..feat
                    }
                } else {
                    feat
                }
            })
            .collect()
    }
}

impl EnsureProperties for FeatureCollection {
    fn ensure_properties(self, name: Option<String>, enum_type: Option<&Type>) -> Self {
        let name = if let Some(n) = name {
            n
        } else {
            "".to_string()
        };
        let length = self.features.len();
        self.into_iter()
            .enumerate()
            .map(|(i, feat)| {
                feat.ensure_properties(
                    Some(if length > 1 {
                        format!("{}_{}", name, i)
                    } else {
                        name.clone()
                    }),
                    enum_type,
                )
            })
            .collect()
    }
}

impl ToMultiVec for FeatureCollection {
    fn to_multi_vec(self) -> MultiVec {
        let mut return_value: MultiVec = vec![];

        for feature in self.into_iter() {
            if let Some(geometry) = feature.geometry {
                match geometry.value {
                    Value::MultiPolygon(_) => geometry
                        .to_feature_vec()
                        .into_iter()
                        .for_each(|f| return_value.push(f.to_single_vec())),
                    Value::GeometryCollection(geometries) => geometries.into_iter().for_each(|g| {
                        let value = g.to_single_vec();
                        if !value.is_empty() {
                            return_value.push(value)
                        }
                    }),
                    _ => return_value.push(geometry.to_single_vec()),
                }
            }
        }
        return_value
    }
}

impl ToSingleStruct for FeatureCollection {
    fn to_single_struct(self) -> single_struct::SingleStruct {
        self.to_single_vec().to_single_struct()
    }
}

impl ToMultiStruct for FeatureCollection {
    fn to_multi_struct(self) -> multi_struct::MultiStruct {
        self.to_multi_vec().to_multi_struct()
    }
}

impl ToText for FeatureCollection {
    fn to_text(self, sep_1: &str, sep_2: &str) -> String {
        self.to_multi_vec().to_text(sep_1, sep_2)
    }
}

impl ToCollection for FeatureCollection {
    fn to_collection(self, _name: Option<String>, _enum_type: Option<&Type>) -> FeatureCollection {
        FeatureCollection {
            bbox: if self.bbox.is_some() {
                self.bbox
            } else {
                self.clone()
                    .into_iter()
                    .flat_map(|x| x.to_single_vec())
                    .collect::<single_vec::SingleVec>()
                    .get_bbox()
            },
            features: self
                .features
                .into_iter()
                .map(|feat| Feature {
                    bbox: if feat.bbox.is_some() {
                        feat.bbox
                    } else {
                        feat.clone().to_single_vec().get_bbox()
                    },
                    ..feat
                })
                .collect(),
            ..self
        }
    }
}

impl ToPoracleVec for FeatureCollection {
    fn to_poracle_vec(self: FeatureCollection) -> Vec<poracle::Poracle> {
        let mut return_vec = vec![];

        for (i, feature) in self.into_iter().enumerate() {
            let mut poracle_feat = poracle::Poracle {
                name: None,
                color: None,
                description: None,
                display_in_matches: None,
                group: None,
                id: None,
                user_selectable: None,
                path: None,
                multipath: None,
            };
            if feature.contains_property("name") {
                poracle_feat.name = Some(
                    feature
                        .property("name")
                        .unwrap()
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                );
            }
            if feature.contains_property("id") {
                poracle_feat.id = Some(PoracleId::Number(
                    feature.property("id").unwrap().as_u64().unwrap_or(i as u64),
                ));
            } else {
                poracle_feat.id = Some(PoracleId::Number(i as u64));
            }
            if feature.contains_property("color") {
                poracle_feat.color = Some(
                    feature
                        .property("color")
                        .unwrap()
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                );
            }
            if feature.contains_property("description") {
                poracle_feat.description = Some(
                    feature
                        .property("description")
                        .unwrap()
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                );
            }
            if feature.contains_property("group") {
                poracle_feat.group = Some(
                    feature
                        .property("group")
                        .unwrap()
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                );
            }
            if feature.contains_property("display_in_matches") {
                poracle_feat.display_in_matches = Some(
                    feature
                        .property("display_in_matches")
                        .unwrap()
                        .as_bool()
                        .unwrap_or(true),
                );
            } else {
                poracle_feat.display_in_matches = Some(true);
            }
            if feature.contains_property("user_selectable") {
                poracle_feat.user_selectable = Some(
                    feature
                        .property("user_selectable")
                        .unwrap()
                        .as_bool()
                        .unwrap_or(true),
                );
            } else {
                poracle_feat.user_selectable = Some(true);
            }
            if let Some(geometry) = feature.geometry {
                let mut multipath: multi_vec::MultiVec = vec![];
                match geometry.value {
                    Value::MultiPolygon(_) => geometry.to_feature_vec().into_iter().for_each(|f| {
                        multipath.push(f.to_single_vec());
                    }),
                    Value::GeometryCollection(geometries) => geometries.into_iter().for_each(|g| {
                        if g.value.type_name() == "Polygon" {
                            let value = g.to_single_vec();
                            if !value.is_empty() {
                                multipath.push(value)
                            }
                        }
                    }),
                    Value::Polygon(_) => poracle_feat.path = Some(geometry.to_single_vec()),
                    _ => {
                        println!(
                            "Poracle format does not support: {:?}",
                            geometry.value.type_name()
                        );
                    }
                }
                if !multipath.is_empty() {
                    poracle_feat.multipath = Some(multipath);
                }
            }
            return_vec.push(poracle_feat);
        }
        return_vec
    }
}
