use error::{Result, SmallestEnclosingH3Error};
use geo::{point, Point};
use h3o::{CellIndex, LatLng, Resolution};

pub mod error;

// in meters
const EARTH_RADIUS: f64 = 6371000.0;

/// Builder for creating a SmallestEnclosingH3 instance
#[derive(Debug)]
pub struct SmallestEnclosingH3Builder {
    resolution: Resolution,
    center: LatLng,
    radius_meters: f64,
}

impl SmallestEnclosingH3Builder {
    pub fn new(center: LatLng, radius_meters: f64, resolution: Resolution) -> Self {
        Self {
            resolution,
            center,
            radius_meters,
        }
    }

    pub fn resolution(mut self, resolution: u8) -> Result<Self> {
        self.resolution = Resolution::try_from(resolution)
            .map_err(|e| SmallestEnclosingH3Error::InvalidResolution(e.to_string()))?;
        Ok(self)
    }

    pub fn center(mut self, center: LatLng) -> Self {
        self.center = center;
        self
    }

    pub fn radius_meters(mut self, radius: f64) -> Result<Self> {
        if radius <= 0.0 {
            return Err(SmallestEnclosingH3Error::InvalidRadius(
                "Radius must be positive".to_string(),
            ));
        }
        self.radius_meters = radius;
        Ok(self)
    }

    pub fn build(self) -> Result<SmallestEnclosingH3> {
        if self.radius_meters <= 0.0 {
            return Err(SmallestEnclosingH3Error::InvalidRadius(
                "Radius must be positive".to_string(),
            ));
        }

        Ok(SmallestEnclosingH3 {
            resolution: self.resolution,
            center: self.center,
            radius_meters: self.radius_meters,
        })
    }
}

/// Represents a collection of H3 hexagons that enclose a circle
#[derive(Debug)]
pub struct SmallestEnclosingH3 {
    resolution: Resolution,
    center: LatLng,
    radius_meters: f64,
}

impl SmallestEnclosingH3 {
    pub fn hexagons(&self) -> Result<Vec<CellIndex>> {
        let center_cell = self.center.to_cell(self.resolution);

        // Calculate the distance to the edge of the circle
        let edge_lat = self.destination_point(
            &point!(x: self.center.lng(), y: self.center.lat()),
            self.radius_meters,
            0.0, // bearing of 0 degrees (north)
        )?;

        let edge_cell = LatLng::new(edge_lat.y(), edge_lat.x())
            .map_err(|e| SmallestEnclosingH3Error::InvalidLatLng(e.to_string()))?
            .to_cell(self.resolution);

        // Calculate the grid distance between center and edge
        let k = center_cell
            .grid_distance(edge_cell)
            .map_err(|e| SmallestEnclosingH3Error::GridDistanceError(e.to_string()))?;

        // Get only the ring at distance k (not the entire disk)
        Ok(center_cell.grid_ring_fast(k as u32).flatten().collect())
    }

    pub fn generate_circle_coordinates(&self) -> Result<Vec<Vec<f64>>> {
        let num_points = 64;
        let center_point = point!(x: self.center.lng(), y: self.center.lat());
        let mut coordinates = Vec::with_capacity(num_points + 1);

        for i in 0..=num_points {
            let bearing = (i as f64 * 360.0 / num_points as f64).to_radians();
            let point = self.destination_point(&center_point, self.radius_meters, bearing)?;
            coordinates.push(vec![point.x(), point.y()]);
        }

        // Close the polygon by repeating the first point
        if let Some(first) = coordinates.first().cloned() {
            coordinates.push(first);
        }

        Ok(coordinates)
    }

    pub fn destination_point(
        &self,
        start: &Point<f64>,
        distance: f64,
        bearing: f64,
    ) -> Result<Point<f64>> {
        let lat1 = start.y().to_radians();
        let lon1 = start.x().to_radians();
        let angular_distance = distance / EARTH_RADIUS;

        let lat2 = (lat1.sin() * angular_distance.cos()
            + lat1.cos() * angular_distance.sin() * bearing.cos())
        .asin();

        let lon2 = lon1
            + (bearing.sin() * angular_distance.sin() * lat1.cos())
                .atan2(angular_distance.cos() - lat1.sin() * lat2.sin());

        Ok(point!(
            x: lon2.to_degrees(),
            y: lat2.to_degrees()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_builder_with_valid_inputs() {
        let center = LatLng::new(0.0, 0.0).unwrap();
        let result = SmallestEnclosingH3Builder::new(center, 1000.0, Resolution::Nine).build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_invalid_radius() {
        let center = LatLng::new(0.0, 0.0).unwrap();
        let result = SmallestEnclosingH3Builder::new(center, -1.0, Resolution::Nine).build();
        assert!(matches!(
            result,
            Err(SmallestEnclosingH3Error::InvalidRadius(_))
        ));
    }

    #[test]
    fn test_matches_python_implementation() {
        // Phoenix center coordinates
        let lat = 33.4484;
        let lng = -112.0740;
        let radius_meters = 50.0;
        let resolution = Resolution::Twelve;

        let center = LatLng::new(lat, lng).unwrap();
        let smallest_enclosing_h3 =
            SmallestEnclosingH3Builder::new(center, radius_meters, resolution)
                .build()
                .unwrap();

        let hexes: HashSet<String> = smallest_enclosing_h3
            .hexagons()
            .unwrap()
            .into_iter()
            .map(|h| h.to_string())
            .collect();

        // From python:
        // mapper = HexagonCircleMapper(resolution=12)
        // phoenix_center = (33.4484, -112.0740)
        // radius = 50
        //
        // # Export hexagon IDs separately for easier comparison
        // hexagons = mapper.get_hexagons(phoenix_center, radius)
        //
        // print(hexagons)
        let python_hexes: HashSet<String> = vec![
            "8c29b6d357aa7ff",
            "8c29b6d357a33ff",
            "8c29b6d357853ff",
            "8c29b6d357ac3ff",
            "8c29b6d357aa5ff",
            "8c29b6d357ab9ff",
            "8c29b6d357aa3ff",
            "8c29b6d357851ff",
            "8c29b6d357ad5ff",
            "8c29b6d357859ff",
            "8c29b6d357a3bff",
            "8c29b6d357acdff",
            "8c29b6d357a13ff",
            "8c29b6d357a17ff",
            "8c29b6d357abdff",
            "8c29b6d357a83ff",
            "8c29b6d357ac1ff",
            "8c29b6d357a8bff",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        assert_eq!(
            hexes.len(),
            python_hexes.len(),
            "Number of hexagons doesn't match"
        );

        let diff: Vec<_> = hexes.symmetric_difference(&python_hexes).collect();

        assert!(
            diff.is_empty(),
            "Hexagon sets don't match. Difference: {:?}",
            diff
        );
    }
}
