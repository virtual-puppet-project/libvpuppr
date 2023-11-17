use godot::prelude::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, GodotClass)]
#[class(init)]
pub struct DataParser;

#[godot_api]
impl DataParser {
    #[func]
    pub fn ifacial_mocap(data: PackedByteArray) -> Dictionary {
        let mut r = Dictionary::new();
        let mut blend_shapes = Dictionary::new();

        match std::str::from_utf8(data.as_slice()) {
            Ok(v) => {
                let mut split = v.split("|");
                while let Some(v) = split.next() {
                    if let Some((k, v)) = v.split_once('#') {
                        // TODO these are all gross, there must be a better way
                        match k {
                            "=head" => {
                                let vals = v.splitn(5, ',').collect::<Vec<&str>>();

                                r.insert(
                                    "rotation",
                                    Vector3::new(
                                        vals.get(0)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                        vals.get(1)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                        vals.get(2)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                    ),
                                );

                                r.insert(
                                    "position",
                                    Vector3::new(
                                        vals.get(3)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                        vals.get(4)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                        vals.get(5)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                    ),
                                );
                            }
                            "rightEye" => {
                                let vals = v.splitn(2, ',').collect::<Vec<&str>>();

                                r.insert(
                                    "right_eye",
                                    Vector3::new(
                                        vals.get(0)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                        vals.get(1)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                        vals.get(2)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                    ),
                                );
                            }
                            "leftEye" => {
                                let vals = v.splitn(2, ',').collect::<Vec<&str>>();

                                r.insert(
                                    "left_eye",
                                    Vector3::new(
                                        vals.get(0)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                        vals.get(1)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                        vals.get(2)
                                            .map(|v| v.parse::<f32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                    ),
                                );
                            }
                            _ => error!("Unhandled ifm data key: {k}"),
                        }
                    } else if let Some((k, v)) = v.split_once("-") {
                        blend_shapes.insert(
                            k
                                // TODO maybe use https://github.com/BurntSushi/aho-corasick for faster replace?
                                .replace("_L", "left")
                                .replace("_R", "right")
                                .to_lowercase(),
                            f32::from(v.parse::<i16>().unwrap_or(0)) / 100.0,
                        );
                    } else if v.is_empty() {
                    } else {
                        error!("Unhandled ifm key-value pair {v}");
                    }
                }
            }
            Err(e) => {
                error!("{e}");
            }
        }

        r.insert("blend_shapes", blend_shapes);
        r
    }

    #[func]
    pub fn vtube_studio(data: PackedByteArray) -> Dictionary {
        let mut r = Dictionary::new();

        #[derive(Debug, Default, Serialize, Deserialize)]
        pub struct VTubeStudioData {
            #[serde(rename = "Rotation")]
            pub rotation: Option<Vector3>,
            #[serde(rename = "Position")]
            pub position: Option<Vector3>,
            #[serde(rename = "EyeLeft")]
            pub eye_left: Option<Vector3>,
            #[serde(rename = "EyeRight")]
            pub eye_right: Option<Vector3>,
            #[serde(rename = "BlendShapes")]
            pub blend_shapes: Option<Vec<VtBlendShape>>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct VtBlendShape {
            pub k: String,
            pub v: f32,
        }

        let data = match serde_json::from_slice::<VTubeStudioData>(data.as_slice()) {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                VTubeStudioData::default()
            }
        };

        r.insert("rotation", data.rotation.unwrap_or_default());
        r.insert("position", data.position.unwrap_or_default());
        r.insert("eye_left", data.eye_left.unwrap_or_default());
        r.insert("eye_right", data.eye_right.unwrap_or_default());
        r.insert(
            "blend_shapes",
            Array::from_iter(data.blend_shapes.unwrap_or_default().into_iter().map(|v| {
                let mut r = Dictionary::new();

                r.insert("k", v.k.to_lowercase());
                r.insert("v", v.v);

                r
            })),
        );

        r
    }
}
