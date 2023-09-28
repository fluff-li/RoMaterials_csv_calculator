use std::{
    fmt,
    fmt::Display,
};


pub struct Part {
    pub name: String,
    pub temp: f32,
    pub height_max: f32,
    pub height_min: f32,
    pub areal_density: f32,
    pub structures: Vec<(Structure, f32)>,
    pub data: Vec<Pair>,
}

#[derive(Clone)]
pub struct Structure {
    pub name: String,
    pub temp: f32,
    pub data: Vec<Pair>,
    pub areal_density: f32,
    pub tickness: f32,
    pub temp_list: Vec<i32>,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Data {
    pub cp: f32,
    pub R_th: f32,
    pub e: f32
}
#[derive(Debug, Clone)]
pub struct Pair(pub f32, pub Data);

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub path: String,
    pub portion: f32,
    pub temp_max: f32,
    pub density: f32,
    pub tickness: f32,
    pub areal_density: f32,
    pub thermal_prop_layer_temp: Vec<Pair>,
    pub thermal_prop_struct_temp: Vec<Pair>,
    pub thermal_prop_struct_temp_frac: Vec<Pair>,
}
impl Default for Layer{
    fn default() -> Self {
        Layer {
            name: "".to_string(),
            path: "".to_string(),
            portion: 0.0,
            temp_max: 0.0,
            density: 0.0,
            tickness: 0.0,
            areal_density: 0.0,
            thermal_prop_layer_temp: Vec::<Pair>::new(),
            thermal_prop_struct_temp: Vec::<Pair>::new(),
            thermal_prop_struct_temp_frac: Vec::<Pair>::new(),
        }
    }
}
impl Display for Layer {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: max Temp {} density {}, tickness {}, portion {}
                    \npath: {} 
                    \n{:?}\n{:?} \n", self.name, self.temp_max, self.density, self.tickness, self.portion, self.path, self.thermal_prop_layer_temp, self.thermal_prop_struct_temp)
    }
}