use crate::{graph::graph::OSMGraph, models::vehicle::Vehicle};
use nanoid::nanoid;
use std::vec::Vec;

#[derive(Debug)]
pub struct VehicleBuilder<'a> {
    pub speed: f64,
    pub path_ids: Vec<usize>,

    pub delta: f64,
    pub is_parked: bool,

    pub prev_id: usize,
    pub next_id: usize,

    pub graph: &'a OSMGraph,
}

impl<'a> VehicleBuilder<'a> {
    pub fn new(graph: &'a OSMGraph) -> VehicleBuilder<'a> {
        VehicleBuilder {
            speed: 0.0,
            path_ids: Vec::new(),
            delta: 0.0,
            is_parked: false,
            prev_id: 0,
            next_id: 0,
            graph,
        }
    }

    pub fn with_speed(mut self, speed: f64) -> VehicleBuilder<'a> {
        self.speed = speed;
        self
    }

    pub fn with_path_ids(mut self, path_ids: Vec<usize>) -> VehicleBuilder<'a> {
        self.path_ids = path_ids;
        self
    }

    // redundant?
    pub fn with_graph(mut self, graph: &'a OSMGraph) -> VehicleBuilder<'a> {
        self.graph = graph;
        self
    }

    pub fn with_prev_id(mut self, prev_id: usize) -> VehicleBuilder<'a> {
        self.prev_id = prev_id;
        self
    }
    pub fn with_next_id(mut self, next_id: usize) -> VehicleBuilder<'a> {
        self.next_id = next_id;
        self
    }

    pub fn with_delta(mut self, delta: f64) -> VehicleBuilder<'a> {
        self.delta = delta;
        self
    }

    pub fn with_is_parked(mut self, is_parked: bool) -> VehicleBuilder<'a> {
        self.is_parked = is_parked;
        self
    }

    pub fn check(&self) -> crate::prelude::Result<()> {
        todo!();
        if self.speed == 0.0 {}
        if self.path_ids.len() < 2 {}
        if self.prev_id < 1 {}

        // TODO: more checks?

        Ok(())
    }

    pub fn from_bytes() -> Vehicle {
        todo!("implement trait to vehicle builder?")
    }

    pub fn build(self) -> crate::prelude::Result<Vehicle> {
        let alphabet: [char; 16] = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
        ];

        let id = nanoid!(10, &alphabet);

        self.check()?;

        Ok(Vehicle {
            id,
            path_ids: self.path_ids,
            speed: self.speed,
            delta: self.delta,
            next_id: self.next_id,
            prev_id: self.prev_id,
            is_parked: self.is_parked,
            distance_remaining: 0.0,
            marked_for_deletion: false,
            graph: self.graph, // FIXME: Graph IDs?
        })
    }
}

//     //TODO rewrite errors
//     fn  check(vb: VehicleBuilder) -> VehicleBuilder {

//         return vb;
//     }

//     fn Build(vb: VehicleBuilder) -> Result<VehicleBuilder, io::Error>{
//         //TODO: error OK?
//         let vb = vb.check()?;
//         Ok(vb);

//         newAlphabet := nanoid.DefaultAlphabet
//         newAlphabet = strings.Replace(newAlphabet, "_", "", -1)
//         newAlphabet = strings.Replace(newAlphabet, "-", "", -1)
//         vid, err := nanoid.Generate(newAlphabet, 20)
//         if err != nil {
//             log.Error().Err(err).Msg("Failed to generate vehicle ID.")
//             return Vehicle{}, err
//         }

//         vehicle := Vehicle{
//             ID:                vid,
//             PathIDs:           vb.pathIDs,
//             Speed:             vb.speed,
//             Delta:             vb.delta,
//             NextID:            vb.nextID,
//             PrevID:            vb.prevID,
//             IsParked:          vb.isParked,
//             DistanceRemaining: 0.0, // default value
//             StreetGraph:       vb.graph,
//         }

//         // ensure nextID is set
//         id := vehicle.GetNextID(vehicle.PathIDs[0])
//         if id == 0 {
//             vehicle.IsParked = true
//             vehicle.NextID = 0
//         } else if id == -1 {
//             // TODO: this should never happen
//             panic("vehicle.GetNextID returned -1 at initialization")
//         } else {
//             vehicle.NextID = id
//         }

//         return vehicle, nil
//     }

// }
