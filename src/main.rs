mod read_write;
mod data_holder;

use read_write::*;
use data_holder::*;

use std::{
    process,
    path::PathBuf, 
    ffi::OsString,
};


// https://docs.rs/csv/latest/csv/tutorial/


const TEMPERATURE_EQUALIZED: f32 = 0.0;//273.15;

const TEMP_LIST: &str = "bib/Temp_List.csv";
const OUTPUT_DIRECTORY: &str = "out/";



fn main() -> std::io::Result<()> {
    let mut structures = Vec::<Structure>::new();

    let structure_paths = get_files("bib/structures".to_string(), OsString::from("csv"));

    for path in structure_paths.iter() {
        let mut structure = read_structure_csv(path);
        structure.temp_list = read_temp_list_csv(&PathBuf::from(TEMP_LIST));

        for layer in structure.layers.iter_mut() {
            read_material_csv(layer);
            adjust_to_geometry(layer);
        }
        create_corresponding_part_temp_list(&mut structure);
    
        calculate_structure(&mut structure);
        output_structure(&structure, OUTPUT_DIRECTORY.to_string() + "structures/");
        structures.push(structure);
        

    }
    let part_paths = get_files("bib/part".to_string(), OsString::from("csv"));


    for path in part_paths.iter() {
        let (mut part, mut part_list) = read_part_csv(path);

        for structure in &structures {
            for (i, (name, portion)) in part_list.iter_mut().enumerate() {
                if structure.name == *name {
                    part.structures.push((structure.clone(), *portion));
                    part_list.remove(i);
                    break;
                }
            }
            if part_list.is_empty() {
                break;
            }
        }
        if !part_list.is_empty() {
            println!("Error Part {}: Structures not found. {:?} ", part.name, part_list );
            process::exit(1);
        }

        calculate_part(&mut part);

        output_part(part, OUTPUT_DIRECTORY.to_string());
    }

    Ok(())
}


/// calculate the part values based on data from its structures
fn calculate_part(part: &mut Part) {
    part.areal_density = 0.0;
    part.height_min = f32::INFINITY;
    part.height_max = 0.0;

    for (structure, portion) in part.structures.iter() {
        part.areal_density += structure.areal_density * portion;

        if part.height_min > structure.tickness {
            part.height_min = structure.tickness;
        }
        if part.height_max < structure.tickness {
            part.height_max = structure.tickness;
        }   
    }

    
    for (i, temp) in part.structures[1].0.temp_list.iter().enumerate() {
        let mut cp = 0.0;
        let mut r_th = 0.0;
        let mut e = 0.0;

        for (structure, portion) in part.structures.iter() {
            cp += structure.data[i].1.cp * structure.areal_density / part.areal_density * portion * (structure.temp-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
            r_th += structure.data[i].1.R_th * portion * (structure.temp-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
            e += structure.data[i].1.e * portion * (structure.temp-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
        }
        part.data.push(Pair((*temp - 25) as f32, Data{cp, R_th: 1.0 / r_th, e: e}));
    }
}

/// calculate the structure values based on data from its layers
fn calculate_structure(structure: &mut Structure) {
    structure.areal_density = 0.0;
    structure.tickness = 0.0;
    
    for layer in structure.layers.iter() {
        structure.areal_density += layer.areal_density; 
        structure.tickness += layer.tickness;
    }
    adjust_to_structure_temp_density(structure);

    for (i, temp) in structure.temp_list.iter().enumerate() {
        let mut cp = 0.0;
        let mut r_th = 0.0;
        let mut e = 0.0;

        for layer in structure.layers.iter() {
            cp += layer.thermal_prop_struct_temp_frac[i].1.cp;
            r_th += layer.thermal_prop_struct_temp_frac[i].1.R_th;

            if e <= 0.0 {
                e = layer.thermal_prop_struct_temp_frac[i].1.e;
            }
        }
        structure.data.push(Pair(*temp as f32, Data{cp, R_th: r_th, e: e}));
    }
}


/// adjust layers value multiplyers based om structure temperature & density
fn adjust_to_structure_temp_density (structure: &mut Structure) {
    for layer in structure.layers.iter_mut() {
        layer.thermal_prop_struct_temp_frac = layer.thermal_prop_struct_temp.clone();

        for (i, prop_list) in layer.thermal_prop_struct_temp_frac.iter_mut().enumerate() {
            prop_list.1.cp *= prop_list.0 / (structure.temp_list[i] as f32 * structure.areal_density) ;
            prop_list.1.R_th *= prop_list.0 / structure.temp_list[i] as f32;
        }
    }
}


/// Copy layer values into new List where the row of layer temperaure is put into the row of corresponding structure Temperature 
fn create_corresponding_part_temp_list(structure: &mut Structure) {
    let temp_range = structure.temp - TEMPERATURE_EQUALIZED;
    let mut corresponding_temps = Vec::<f32>::new();
    let mut corresponding_mults = Vec::<f32>::new();

    for layer in structure.layers.iter_mut() {
        for temp in structure.temp_list.iter() {
            let temp_correspond = (layer.temp_max - TEMPERATURE_EQUALIZED) / temp_range * (*temp as f32 - TEMPERATURE_EQUALIZED) + TEMPERATURE_EQUALIZED;
            corresponding_temps.push(temp_correspond);
            corresponding_mults.push(temp_correspond / *temp as f32);
        }
        layer.thermal_prop_struct_temp = adjust_list(&layer, &corresponding_temps);
        corresponding_temps.clear();
        corresponding_mults.clear();
    }
}


/// Copy layer values into new List where the row of layer temperaure is put into the row of corresponding structure Temperature 
fn adjust_list(layer: & Layer, temp_coresponding: & Vec<f32>) -> Vec<Pair>{

    let mut data_adjusted: Vec<Pair> = Vec::<Pair>::new();

    let mut n = 0;
    let mut pushed_n1 = false;
    let mut pushed_0 = false;
    data_adjusted.push(Pair(0.0, Data{cp: 0.0, R_th: 0.0, e: 0.0}));

    // find the corresponding row in the new list & copy values
    for j in 1..temp_coresponding.len() {
        
        for i in n..layer.thermal_prop_layer_temp.len() { 
            if layer.thermal_prop_layer_temp[i].0 > temp_coresponding[j-1] && layer.thermal_prop_layer_temp[i].0 < temp_coresponding[j] {
                if ( temp_coresponding[j-1] - layer.thermal_prop_layer_temp[i].0).abs() > (temp_coresponding[j] - layer.thermal_prop_layer_temp[i].0).abs() {
                    data_adjusted.push(layer.thermal_prop_layer_temp[i].clone());
                    pushed_0 = true;
                    n = i+1;
                    break;
                } else if !pushed_n1 {
                    if data_adjusted.len() > j-1 {
                        data_adjusted[j-1] = layer.thermal_prop_layer_temp[i].clone();
                    } else {
                        data_adjusted.push(layer.thermal_prop_layer_temp[i].clone())
                    }
                    n = i+1;
                    break;
                }
            } else if layer.thermal_prop_layer_temp[i].0 < temp_coresponding[j-1] && j >= 2 {
                if ( temp_coresponding[j-1] - layer.thermal_prop_layer_temp[i].0).abs() > (temp_coresponding[j-2] - layer.thermal_prop_layer_temp[i].0).abs() {
                    data_adjusted.push(layer.thermal_prop_layer_temp[i].clone());
                    pushed_0 = true;
                    n = i+1;
                    break;
                } else if !pushed_n1 {
                    if data_adjusted.len() > j-1 {
                        data_adjusted[j-1] = layer.thermal_prop_layer_temp[i].clone();
                    } else {
                        data_adjusted.push(layer.thermal_prop_layer_temp[i].clone())
                    }
                    n = i+1;
                    break;
                }
            }
        }
        if !pushed_0 {
            data_adjusted.push(Pair(0.0, Data{cp: 0.0, R_th: 0.0, e: 0.0}));
        } else {
            pushed_0 = false;
            pushed_n1 = true;
        }
    }

    let mut i = 0;

    // fill gaps up to first entry, copying values of the first filled entry
    while i < data_adjusted.len() {
        if data_adjusted[i].0 != 0.0 {
            if i > 0 {
                for j in 0..i {
                    data_adjusted[j].0 = data_adjusted[i].0;
                    data_adjusted[j].1.cp = data_adjusted[i].1.cp;
                    data_adjusted[j].1.R_th  = data_adjusted[i].1.R_th;
                    data_adjusted[j].1.e  = data_adjusted[i].1.e;
                } 
            }
            i += 1;
            break;
        }
        i += 1;
    }

    let mut lowerbound  = 1001;
    let mut upperbound = 1001;
    let mut last_entry= data_adjusted.len();

    // fill all the gaps between existing entries, the values are linear function between those entries
    while i < data_adjusted.len() {
        if data_adjusted[i].0 == 0.0 {
            if lowerbound > 1000 {
                lowerbound = i - 1;
                upperbound = i;
            }
        } else {
            last_entry = i;
            if upperbound < 1000 {
                upperbound = i;

                let temp_delta = data_adjusted[upperbound].0 - data_adjusted[lowerbound].0; 
                let cp_fac = (data_adjusted[upperbound].1.cp - data_adjusted[lowerbound].1.cp) / temp_delta;
                let k_fac = (data_adjusted[upperbound].1.R_th - data_adjusted[lowerbound].1.R_th) / temp_delta;
                let e_fac = (data_adjusted[upperbound].1.e - data_adjusted[lowerbound].1.e) / temp_delta;
                

                for n in lowerbound+1..upperbound {
                    let temp_delta2 = temp_coresponding[n] - data_adjusted[lowerbound].0;
                    data_adjusted[n].0 = temp_delta2 + data_adjusted[lowerbound].0;
                    data_adjusted[n].1.cp = cp_fac * temp_delta2 + data_adjusted[lowerbound].1.cp;
                    data_adjusted[n].1.R_th = k_fac * temp_delta2 + data_adjusted[lowerbound].1.R_th;
                    data_adjusted[n].1.e = e_fac * temp_delta2 + data_adjusted[lowerbound].1.e;
                }
                lowerbound = 1001;
                upperbound = 1001;
            }
        }
        i += 1;
    }

    // fill the rest by copying the last filled entry 
    for n in last_entry..data_adjusted.len() {
        data_adjusted[n].0 = data_adjusted[last_entry].0;
        data_adjusted[n].1.cp = data_adjusted[last_entry].1.cp;
        data_adjusted[n].1.R_th  = data_adjusted[last_entry].1.R_th;
        data_adjusted[n].1.e  = data_adjusted[last_entry].1.e;
    }
    data_adjusted
}


/// Adjust read values to thickness & density
fn adjust_to_geometry (layer: &mut Layer ) {
    layer.areal_density = layer.density * layer.tickness;
    for prop_list in layer.thermal_prop_layer_temp.iter_mut() {
        prop_list.1.cp *= layer.areal_density;
        prop_list.1.R_th = layer.tickness / prop_list.1.R_th * 1000.0;
    }
}
