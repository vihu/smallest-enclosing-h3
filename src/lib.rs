use error::{Result, SmallestEnclosingH3Error};
use geo::algorithm::vincenty_distance::VincentyDistance;
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
        let k = self.calculate_k_ring_size()?;
        Ok(center_cell.grid_disk_fast(k).flatten().collect())
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

    fn calculate_k_ring_size(&self) -> Result<u32> {
        let center_cell = self.center.to_cell(self.resolution);
        let center_point = point!(x: self.center.lng(), y: self.center.lat());

        // Get the hex edge length
        let hex_edge = self.resolution.edge_length_m();

        // Calculate number of rings needed to cover the radius
        // Adding a small buffer to ensure we fully cover the circle
        let k = (self.radius_meters / hex_edge).ceil() as u32;

        // TODO: Verify the calculation by checking the actual distance
        let ring_cells = center_cell.grid_disk_fast(k).flatten();
        let mut max_distance: f64 = 0.0;

        for cell in ring_cells {
            if let Some(cell_center) = cell.center_child(Resolution::Fifteen) {
                let latlng = LatLng::from(cell_center);
                let cell_point = point!(x: latlng.lng(), y: latlng.lat());
                let distance = center_point.vincenty_distance(&cell_point).unwrap_or(0.0);
                max_distance = max_distance.max(distance);
            }
        }

        if max_distance < self.radius_meters {
            Ok(k + 1)
        } else {
            Ok(k)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
