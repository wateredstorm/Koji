use super::*;

impl EnsurePoints for Feature {
    fn ensure_first_last(self) -> Self {
        let geometry = if let Some(geometry) = self.geometry {
            Some(geometry.ensure_first_last())
        } else {
            None
        };
        Self { geometry, ..self }
    }
}

impl FeatureHelpers for Feature {
    fn add_instance_properties(&mut self, name: Option<String>, enum_type: Option<&Type>) {
        if !self.contains_property("name") {
            if let Some(name) = name {
                self.set_property("name", name)
            }
        }
        if !self.contains_property("type") {
            if let Some(enum_type) = enum_type {
                self.set_property("type", enum_type.to_string());
                // match enum_type {
                //     Type::CirclePokemon | Type::CircleSmartPokemon => {
                //         self.set_property("radius", 70);
                //     }
                //     Type::CircleRaid | Type::CircleSmartRaid => {
                //         self.set_property("radius", 700);
                //     }
                //     Type::ManualQuest => {
                //         self.set_property("radius", 80);
                //     }
                //     _ => {}
                // }
            } else if let Some(geometry) = self.geometry.as_ref() {
                match geometry.value {
                    Value::Point(_) | Value::MultiPoint(_) => {
                        self.set_property("type", "CirclePokemon");
                    }
                    Value::Polygon(_) | Value::MultiPolygon(_) => {
                        self.set_property("type", "AutoQuest");
                    }
                    _ => {}
                }
            }
        }
    }
    fn remove_last_coord(self) -> Self {
        if let Some(geometry) = self.geometry {
            let geometry = match geometry.value {
                Value::MultiPoint(value) => {
                    let mut new_value = value;
                    new_value.pop();
                    Geometry {
                        value: Value::MultiPoint(new_value),
                        ..geometry
                    }
                }
                _ => geometry,
            };
            Self {
                geometry: Some(geometry),
                ..self
            }
        } else {
            self
        }
    }
}

impl EnsureProperties for Feature {
    fn ensure_properties(self, name: Option<String>, enum_type: Option<&Type>) -> Self {
        let mut mutable_self = self;
        mutable_self.add_instance_properties(name, enum_type);
        mutable_self
    }
}

impl ToSingleVec for Feature {
    fn to_single_vec(self) -> single_vec::SingleVec {
        self.to_multi_vec().into_iter().flatten().collect()
    }
}

impl ToMultiVec for Feature {
    fn to_multi_vec(self) -> multi_vec::MultiVec {
        let mut return_value = vec![];
        if let Some(geometry) = self.geometry {
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
        return_value
    }
}

impl ToText for Feature {
    fn to_text(self, sep_1: &str, sep_2: &str) -> String {
        self.to_multi_vec().to_text(sep_1, sep_2)
    }
}

impl ToFeatureVec for Feature {
    fn to_feature_vec(self) -> Vec<Feature> {
        if let Some(geometry) = self.geometry {
            geometry.to_feature_vec()
        } else {
            vec![self]
        }
    }
}

impl ToCollection for Feature {
    fn to_collection(self, name: Option<String>, enum_type: Option<&Type>) -> FeatureCollection {
        let bbox = if self.bbox.is_some() {
            self.bbox
        } else {
            self.clone().to_single_vec().get_bbox()
        };
        FeatureCollection {
            bbox: bbox.clone(),
            features: vec![Feature { bbox, ..self }
                .ensure_first_last()
                .ensure_properties(name, enum_type)],
            foreign_members: None,
        }
    }
}

impl ToCollection for Vec<Feature> {
    fn to_collection(self, name: Option<String>, enum_type: Option<&Type>) -> FeatureCollection {
        let name = if let Some(name) = name {
            name
        } else {
            "".to_string()
        };
        let length = self.len();
        FeatureCollection {
            bbox: self
                .clone()
                .into_iter()
                .flat_map(|feat| feat.to_single_vec())
                .collect::<single_vec::SingleVec>()
                .get_bbox(),
            features: self
                .into_iter()
                .enumerate()
                .map(|(i, feat)| {
                    feat.ensure_first_last().ensure_properties(
                        Some(if length > 1 {
                            format!("{}_{}", name, i)
                        } else {
                            name.clone()
                        }),
                        enum_type,
                    )
                })
                .collect(),
            foreign_members: None,
        }
    }
}