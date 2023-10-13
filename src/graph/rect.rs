use crate::models::graph_input::Vertex;
use crate::prelude::*;

// Define a Point struct to represent a 2D point with x and y coordinates
#[derive(Debug, Clone)]
pub(crate) struct Point {
    pub x: f64,
    pub y: f64,
}

// Define a Rect struct to represent a rectangle, which has two points (top_right and bottom_left) and a list of vertices
#[derive(Debug, Clone)]
pub(crate) struct Rect {
    pub top_right: Point,
    pub bottom_left: Point,
    pub vertices: Vec<Vertex>,
}

/// Implementation block for Rect
impl Rect {
    pub fn new(vertices: Vec<Vertex>) -> Result<Rect> {
        if vertices.is_empty() {
            return Err(Error::EmptyVector(String::from(
                "Vertices vector is empty. Cannot create a rectangle without vertices.",
            )));
        }

        // Create a new Rect with default points and vertices and then set the actual points
        let mut rr = Rect {
            bottom_left: Point { x: 0.0, y: 0.0 },
            top_right: Point { x: 0.0, y: 0.0 },
            vertices,
        };
        rr = rr.set_top_right_bottom_left();

        Ok(rr)
    }

    // Determines if a vertex is inside the rectangle based on its x value only
    pub fn in_rect(&self, v: Vertex) -> bool {
        let x = v.x;
        let buffer = 1e-9;
        self.bottom_left.x - buffer <= x && x < self.top_right.x + buffer
    }

    // Function to set the top_right and bottom_left points based on the vertices
    pub fn set_top_right_bottom_left(mut self) -> Self {
        let vtx_lst = self.vertices.clone();

        // Initialize variables to find the extreme points
        let mut bot_y = f64::MAX;
        let mut bot_x = f64::MAX;
        let mut top_x = 0.0;
        let mut top_y = 0.0;

        // Iterate over the vertices to find the extreme points
        for vertex in vtx_lst {
            if vertex.x < bot_x {
                bot_x = vertex.x;
            }
            if vertex.y < bot_y {
                bot_y = vertex.y;
            }
            if vertex.x > top_x {
                top_x = vertex.x;
            }
            if vertex.y > top_y {
                top_y = vertex.y;
            }
        }

        // Update the Rect's points with the extreme points found
        self.bottom_left = Point { x: bot_x, y: bot_y };
        self.top_right = Point { x: top_x, y: top_y };

        self
    }
}

// Unit tests for the Rect struct
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_rect() {
        // Create a rectangle with vertices
        let rect = Rect::new(vec![
            Vertex {
                x: 1.0,
                y: 1.0,
                osm_id: 1,
            },
            Vertex {
                x: 2.0,
                y: 2.0,
                osm_id: 2,
            },
            Vertex {
                x: 3.0,
                y: 3.0,
                osm_id: 3,
            },
        ])
        .unwrap();

        // Test whether a vertex is inside the rectangle
        assert!(rect.in_rect(Vertex {
            x: 1.0,
            y: 1.0,
            osm_id: 1
        }));
        // Test whether a vertex is outside the rectangle
        assert!(!rect.in_rect(Vertex {
            x: 4.0,
            y: 4.0,
            osm_id: 4
        }));
    }

    #[test]
    fn test_set_top_right_bottom_left() {
        // Create a rectangle with vertices
        let mut rect = Rect::new(vec![
            Vertex {
                x: 1.0,
                y: 1.0,
                osm_id: 1,
            },
            Vertex {
                x: 2.0,
                y: 2.0,
                osm_id: 2,
            },
            Vertex {
                x: 3.0,
                y: 3.0,
                osm_id: 3,
            },
        ])
        .unwrap();

        // Call the function to set top_right and bottom_left
        rect = rect.set_top_right_bottom_left();

        // Assert that the points have been set correctly
        assert_eq!(rect.bottom_left.x, 1.0);
        assert_eq!(rect.bottom_left.y, 1.0);
        assert_eq!(rect.top_right.x, 3.0);
        assert_eq!(rect.top_right.y, 3.0);
    }
}
