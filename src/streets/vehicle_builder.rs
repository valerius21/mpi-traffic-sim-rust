use crate::{graph::graph::OSMGraph, models::vehicle::Vehicle};
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

    pub fn from_bytes() {
        todo!()
    }

    pub fn check(&self) -> Result<(), ()> {
        todo!()
    }

    pub fn build(self) -> Vehicle {
        todo!()
    }
}

//     //TODO rewrite errors
//     fn  check(vb: VehicleBuilder) -> VehicleBuilder {
//         if vb.speed == 0. {
//             err = errors.New("speed is not set");
//             log.Error().Err(err).Msg("Failed to build vehicle.");
//             return err
//         }
//         if vb.path_ids.len() < 2 {
//             err = errors.New("path is not set");
//             log.Error().Err(err).Msg("Failed to build vehicle.");
//             return err
//         }
//         if vb.graph == nil {
//             err = errors.New("graph is not set");
//             log.Error().Err(err).Msg("Failed to build vehicle.");
//             return err
//         }
//         if vb.prev_id < 1 {
//             err = errors.New("prevID is not set");
//             log.Error().Err(err).Msg("Failed to build vehicle.");
//             return err
//         }
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
