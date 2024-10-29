use crate::{Result, SmallestEnclosingH3};
use serde_json::Map;

#[cfg(feature = "geojson_export")]
pub mod geojson_export {
    use super::*;
    use geojson::{Feature, FeatureCollection, Geometry, Value};
    use h3o::CellIndex;
    use serde_json::json;

    impl SmallestEnclosingH3 {
        pub fn to_geojson(&self) -> Result<FeatureCollection> {
            let mut features = Vec::new();

            // Add hexagon features
            for hex_id in self.hexagons()? {
                features.push(create_hex_feature(hex_id)?);
            }

            // Add circle feature
            features.push(create_circle_feature(
                self.generate_circle_coordinates()?,
                self.radius_meters,
            )?);

            Ok(FeatureCollection {
                features,
                bbox: None,
                foreign_members: None,
            })
        }
    }

    fn create_circle_feature(coordinates: Vec<Vec<f64>>, radius_meters: f64) -> Result<Feature> {
        let mut properties = Map::new();
        properties.insert("type".to_string(), json!("circle"));
        properties.insert("radius_meters".to_string(), json!(radius_meters));

        Ok(Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Polygon(vec![coordinates]))),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        })
    }

    fn create_hex_feature(hex_id: CellIndex) -> Result<Feature> {
        let boundary: Vec<_> = hex_id
            .boundary()
            .iter()
            .map(|p| vec![p.lng(), p.lat()])
            .collect();

        // Close the polygon by repeating the first point
        let mut coordinates = vec![boundary];
        // Get the first point before mutating the vector
        if let Some(first) = coordinates[0].first().cloned() {
            coordinates[0].push(first);
        }

        let mut properties = Map::new();
        properties.insert("hex_id".to_string(), json!(hex_id.to_string()));
        properties.insert("type".to_string(), json!("hexagon"));

        Ok(Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Polygon(coordinates))),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use h3o::{LatLng, Resolution};
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_geojson_export() {
        // Phoenix coordinates from Python example
        let lat = 33.4484;
        let lng = -112.0740;
        let radius_meters = 50.0;
        let resolution = Resolution::Twelve;

        let center = LatLng::new(lat, lng).unwrap();
        let smallest_enclosing_h3 =
            crate::SmallestEnclosingH3Builder::new(center, radius_meters, resolution)
                .build()
                .unwrap();

        let geojson = smallest_enclosing_h3.to_geojson().unwrap();

        // Basic structure checks
        assert!(geojson.features.len() > 1, "Should have multiple features");

        // Check that we have both hexagons and a circle
        let hexagon_count = geojson
            .features
            .iter()
            .filter(|f| f.property("type").and_then(|v| v.as_str()) == Some("hexagon"))
            .count();

        let circle_count = geojson
            .features
            .iter()
            .filter(|f| f.property("type").and_then(|v| v.as_str()) == Some("circle"))
            .count();

        assert!(hexagon_count > 0, "Should have hexagon features");
        assert_eq!(circle_count, 1, "Should have exactly one circle feature");

        // Optional: Write to file for visual inspection
        let json_string = serde_json::to_string_pretty(&geojson).unwrap();
        File::create("rust_map.geojson")
            .unwrap()
            .write_all(json_string.as_bytes())
            .unwrap();
    }
}
