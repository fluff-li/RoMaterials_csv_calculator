use std::{
    fmt,
    fmt::Display,
    ops::*,
};

pub struct Part {
    pub name: String,
    pub description: String,
    pub temp: f32,
    pub absorbation_const: f32,
    pub cost_per_area: f32,
    pub has_ablator: bool,
    pub height_min1: f32,
    pub height_min0: f32,
    pub height_max1: f32,
    pub height_max0: f32,
    pub areal_density_min: f32,
    pub areal_density_max: f32,
    pub tps_list_min: Vec<(TPS, f32, Vec<DataTriplet>)>,
    pub tps_list_max: Vec<(TPS, f32, Vec<DataTriplet>)>,
    pub data_min: Vec<DataPair>,
    pub data_max: Vec<DataPair>,
}

#[derive(Clone)]
pub struct TPS {
    pub name: String,
    pub temp_max: f32,
    pub data: Vec<DataPair>,
    pub areal_density: f32,
    pub tickness: f32,
    pub temp_list2: Vec<f32>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub name: String,
    pub path: String,
    pub portion: f32,
    pub temp_max: f32,
    pub temp_hot_side: f32,
    pub temp_cold_side: f32,
    pub density: f32,
    pub tickness: f32,
    pub areal_density: f32,
    pub data_csv: Vec<DataPair>,
    pub data_height_adjust: Vec<DataTriplet>,
    pub data_tps_temp_map: Vec<DataTriplet>,
    pub data_tps_temp_mult: Vec<DataTriplet>,
    pub data_avg_r: Vec<DataTriplet>,
}
impl Default for Segment{
    fn default() -> Self {
        Segment {
            name: "".to_string(),
            path: "".to_string(),
            portion: 0.0,
            temp_max: 0.0,
            temp_hot_side: 0.0,
            temp_cold_side: 0.0,
            density: 0.0,
            tickness: 0.0,
            areal_density: 0.0,
            data_csv: Vec::<DataPair>::new(),
            data_height_adjust: Vec::<DataTriplet>::new(),
            data_tps_temp_map: Vec::<DataTriplet>::new(),
            data_tps_temp_mult: Vec::<DataTriplet>::new(),
            data_avg_r: Vec::<DataTriplet>::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct Data {
    pub cp: f32,
    pub R_th: f32,
    pub e: f32
}
impl Add<Data> for Data {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Data{
            cp: self.cp + other.cp,
            R_th: self.R_th + other.R_th,
            e: self.e + other.e,
        }
    }

}
impl Sub<Data> for Data {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Data{
            cp: self.cp - other.cp,
            R_th: self.R_th - other.R_th,
            e: self.e - other.e,
        }
    }

}
impl Div<Data> for Data {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        Data{
            cp: self.cp / other.cp,
            R_th: self.R_th / other.R_th,
            e: self.e / other.e,
        }
    }

}
impl Div<f32> for Data {
    type Output = Self;
    fn div(self, other: f32) -> Self::Output {
        Data{
            cp: self.cp / other,
            R_th: self.R_th / other,
            e: self.e / other,
        }
    }

}
impl Mul<f32> for Data {
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        Data{
            cp: self.cp * other,
            R_th: self.R_th * other,
            e: self.e * other,
        }
    }

}
#[derive(Debug, Clone, Copy)]
pub struct DataPair(pub f32, pub Data);
impl DataPair {
    pub fn to_data_triplet(self) -> DataTriplet {
        DataTriplet{temp_part:0.0, thermal_data: self.1, temp_sub_part: self.0}
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DataTriplet {
    pub temp_part: f32, 
    pub thermal_data: Data, 
    pub temp_sub_part: f32
}
impl DataTriplet {
    pub fn to_data_pair(self) -> DataPair {
        DataPair(self.temp_part, self.thermal_data)
    }
}
