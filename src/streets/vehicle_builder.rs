//TODO: package/imports

use crate::graph::graph::OSMGraph;
use std::vec::Vec;

#[derive(Default, Debug)]
pub struct VehicleBuilder {
    pub speed:   f64,
	pub path_ids: Vec<usize>, 
 
	pub delta:    f64,
	pub is_parked: bool,

	pub prev_id: usize,
	pub next_id: usize,
    //FIXME: Add Graph
}

impl VehicleBuilder {
    pub fn new() -> VehicleBuilder {
        VehicleBuilder {
            ..Default::default()
        }
    }

    pub fn with_speed(mut self, speed: f64) -> VehicleBuilder {
        self.speed = speed;
        self
    }

    pub fn with_path_ids(mut self, path_ids: Vec<usize>) -> VehicleBuilder {
        self.path_ids = path_ids;
        self
    }

    pub fn with_graph(mut self, graph: &OSMGraph) -> VehicleBuilder {
        todo!("not implemented");
    }

    // pub fn build(self) -> Vehicle 
}


// impl VehicleBuilder  {

//     pub fn new_vehicle_builder() -> VehicleBuilder {
//         VehicleBuilder { speed: (), path_ids: (), delta: (), is_parked: (), prev_id: (), next_id: () }
//     }

//     pub fn with_speed(&self, speed: f64) -> VehicleBuilder {
//         self.speed = speed;
//         return VehicleBuilder { speed: (), path_ids: (), delta: (), is_parked: (), prev_id: (), next_id: (), graph: () }
//     }

//     pub fn with_path_ids(&self, path_ids: &[i32]) -> VehicleBuilder {
//         self.path_ids = path_ids;
//         return VehicleBuilder;
//     }

//     //TODO streetgraph reference
//     pub fn with_graph(&self, graph: *const StreetGraph) -> VehicleBuilder {
//         self.graph = graph;
//         return VehicleBuilder;
//     }
    
//     pub fn with_last_id(&self, last_id: i32) -> VehicleBuilder {
//         self.prev_id = last_id;
//         return VehicleBuilder;
//     }
    
//     pub fn with_next_id(&self, next_id: i32) -> VehicleBuilder {
//         self.next_id = next_id;
//         return VehicleBuilder;
//     }
    
//     pub fn with_delta(&self, delta: f64) -> VehicleBuilder {
//         self.delta = delta;
//         return VehicleBuilder;
//     }
    
//     pub fn with_is_parked(&self, is_parked: bool) -> VehicleBuilder {
//         self.is_parked = is_parked;
//         return VehicleBuilder;
//     }
    
//     pub fn from_json_bytes(&self, json_bytes: &[Byte]) -> Result<VehicleBuilder, io::Error> {
       
//         let v = UnmarshalVehicle(json_bytes)?;
//         Ok(v);
//         //TODO: cehck if error handling OK
    
//         self.speed = v.speed;
//         self.path_ids = v.path_ids;
//         self.delta = v.delta;
//         self.is_parked = v.is_parked;
//         self.prev_id = v.prev_id;
//         self.next_id = v.next_id;
    
//         return v;
//     }
    
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

