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
            return Err(Error::EmptyVector(String::from("Empty Vector")));
        }

        let mut rr = Rect {
            bottom_left: Point { x: 0.0, y: 0.0 },
            top_right: Point { x: 0.0, y: 0.0 },
            vertices,
        };
        rr = rr.set_top_right_bottom_left();

        Ok(rr)
    }

    pub fn in_rect(&self, v: Vertex) -> bool {
        let x = v.x;
        let y = v.y;

        self.bottom_left.x <= x
            && x <= self.top_right.x
            && self.bottom_left.y <= y
            && y <= self.top_right.y
    }

    pub fn set_top_right_bottom_left(mut self) -> Self {
        let vtx_lst = self.vertices.clone();

        // set to oppisite sites
        let mut bot_y = 100.0;
        let mut bot_x = 100.0;
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
