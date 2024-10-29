# smallest-enclosing-h3

A Rust library to find the smallest set of H3 hexagons that fully enclose a circle of given radius.

## Usage

```rust
use h3o::LatLng;
use smallest_enclosing_h3::SmallestEnclosingH3Builder;

fn main() {
    // Create a center point (latitude, longitude)
    let center = LatLng::new(33.4484, -112.0740).unwrap();

    // Initialize with center, radius (in meters), and H3 resolution
    let hex_circle = SmallestEnclosingH3Builder::new(center, 500.0, Resolution::Nine)
        .build()
        .unwrap();

    // Get the H3 hexagon indices that enclose the circle
    let hexagons = hex_circle.hexagons().unwrap();
}
```
