use crate::models::graph_input::Vertex;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub(crate) struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct Rect {
    pub top_right: Point,
    pub bottom_left: Point,
    pub vertices: Vec<Vertex>,
}

impl Rect {
    pub fn new(vertices: Vec<Vertex>) -> Result<Rect> {
        if vertices.is_empty() {
            return Err(Error::EmptyVector(String::from(
                "Vertices vector is empty. Cannot create a rectangle without vertices.",
            )));
        }

        let mut rr = Rect {
            bottom_left: Point { x: 0.0, y: 0.0 },
            top_right: Point { x: 0.0, y: 0.0 },
            vertices,
        };
        rr = rr.set_top_right_bottom_left();

        Ok(rr)
    }

    // NOTE: Determines if a vertex is in a rect by x value only
    pub fn in_rect(&self, v: Vertex) -> bool {
        let x = v.x;
        let buffer = 1e-9;
        self.bottom_left.x - buffer <= x && x < self.top_right.x + buffer
    }

    pub fn set_top_right_bottom_left(mut self) -> Self {
        let vtx_lst = self.vertices.clone();

        // set to oppisite sites
        let mut bot_y = f64::MAX;
        let mut bot_x = f64::MAX;
        let mut top_x = 0.0;
        let mut top_y = 0.0;

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

        self.bottom_left = Point { x: bot_x, y: bot_y };
        self.top_right = Point { x: top_x, y: top_y };

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_rect() {
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

        assert!(rect.in_rect(Vertex {
            x: 1.0,
            y: 1.0,
            osm_id: 1
        }));
        assert!(!rect.in_rect(Vertex {
            x: 4.0,
            y: 4.0,
            osm_id: 4
        }));
    }

    #[test]
    fn test_set_top_right_bottom_left() {
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

        rect = rect.set_top_right_bottom_left();

        assert_eq!(rect.bottom_left.x, 1.0);
        assert_eq!(rect.bottom_left.y, 1.0);
        assert_eq!(rect.top_right.x, 3.0);
        assert_eq!(rect.top_right.y, 3.0);
    }
}
