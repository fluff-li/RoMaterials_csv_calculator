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
const TEMP_STEP: f32 = 50.0;
const TEMP_LIST: &str = "bib/Temp_List.csv";
const OUTPUT_DIRECTORY: &str = "out/";



fn main() -> std::io::Result<()> {
    let mut structures = Vec::<Structure>::new();

    let structure_paths = get_files("bib/structures".to_string(), OsString::from("csv"));

    for path in structure_paths.iter() {
        let mut structure = read_structure_csv(path);
        structure.temp_list2 = read_temp_list_csv2(&PathBuf::from(TEMP_LIST));

        for layer in structure.layers.iter_mut() {
            read_material_csv(layer);
            fill_gaps_in_csv(&mut layer.thermal_prop_layer_temp);   
            match output_data_Pair(&layer.name, &layer.thermal_prop_layer_temp, "out/test/layers".to_string()) {
                Ok(result) => {result},
                Err(err) =>  {println!("Error on output_data_Pair {}", err);
                                    process::exit(1);}
            }; 
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

        output_part(part, OUTPUT_DIRECTORY.to_string() + "parts/");
    }

    Ok(())
}

fn fill_gaps_in_csv(thermal_list: &mut Vec<DataPair>,) {
    let mut lower_bound = usize::MAX;
    let mut upper_bound = usize::MAX;
    
    let mut n_cp = usize::MAX;
    let mut n_r_th = usize::MAX;
    let mut n_e = usize::MAX;


    // copy first non zero entry to all entries before
    for i in 0..thermal_list.len() {
        if thermal_list[i].1.cp != 0.0 && n_cp == usize::MAX {
            n_cp = i;
            for j in 0..i {
                thermal_list[j].1.cp = thermal_list[i].1.cp 
            }
        }
        if thermal_list[i].1.R_th != 0.0 && n_r_th == usize::MAX {
            n_r_th = i;
            for j in 0..i {
                thermal_list[j].1.R_th = thermal_list[i].1.R_th 
            }
        }
        if thermal_list[i].1.e != 0.0 && n_e == usize::MAX {
            n_e = i;
            for j in 0..i {
                thermal_list[j].1.e = thermal_list[i].1.e 
            }
        }
        if n_cp != usize::MAX && n_r_th != usize::MAX && n_e != usize::MAX {
            break;
        }
    }

    // Fill the gaps inbetween
    for i in n_cp..thermal_list.len() {
        if thermal_list[i].1.cp == 0.0 && lower_bound == usize::MAX && i > 0{
            lower_bound = i-1;
            n_cp = lower_bound;
        }
        if thermal_list[i].1.cp != 0.0 && lower_bound != usize::MAX {
            upper_bound = i;
        }
        if upper_bound != usize::MAX && lower_bound != usize::MAX {
            let temp_delta = thermal_list[upper_bound].0 - thermal_list[lower_bound].0;
            let data_delta = thermal_list[upper_bound].1.cp - thermal_list[lower_bound].1.cp;

            for j in (lower_bound + 1)..upper_bound {
                thermal_list[j].1.cp = data_delta / temp_delta * (thermal_list[j].0 - thermal_list[lower_bound].0) + thermal_list[lower_bound].1.cp;
            }
            upper_bound = usize::MAX;
            lower_bound = usize::MAX;
        }
    }

    upper_bound = usize::MAX;
    lower_bound = usize::MAX;
    for i in n_r_th..thermal_list.len() {
        if thermal_list[i].1.R_th == 0.0 && lower_bound == usize::MAX && i > 0{
            lower_bound = i-1;
            n_r_th = lower_bound;
        }
        if thermal_list[i].1.R_th != 0.0 && lower_bound != usize::MAX {
            upper_bound = i;
        }
        if upper_bound != usize::MAX && lower_bound != usize::MAX {
            let temp_delta = thermal_list[upper_bound].0 - thermal_list[lower_bound].0;
            let data_delta = thermal_list[upper_bound].1.R_th - thermal_list[lower_bound].1.R_th;

            for j in (lower_bound + 1)..upper_bound {
                thermal_list[j].1.R_th = data_delta / temp_delta * (thermal_list[j].0 - thermal_list[lower_bound].0) + thermal_list[lower_bound].1.R_th;
            }
            upper_bound = usize::MAX;
            lower_bound = usize::MAX;
        }
    }

    upper_bound = usize::MAX;
    lower_bound = usize::MAX;
    for i in n_e..thermal_list.len() {
        if thermal_list[i].1.e == 0.0 && lower_bound == usize::MAX && i > 0{
            lower_bound = i-1;
            n_e = lower_bound;
        }
        if thermal_list[i].1.e != 0.0 && lower_bound != usize::MAX {
            upper_bound = i;
        }
        if upper_bound != usize::MAX && lower_bound != usize::MAX {
            let temp_delta = thermal_list[upper_bound].0 - thermal_list[lower_bound].0;
            let data_delta = thermal_list[upper_bound].1.e - thermal_list[lower_bound].1.e;

            for j in (lower_bound + 1)..upper_bound {
                thermal_list[j].1.e = data_delta / temp_delta * (thermal_list[j].0 - thermal_list[lower_bound].0) + thermal_list[lower_bound].1.e;
            }
            upper_bound = usize::MAX;
            lower_bound = usize::MAX;
        }
    }

    // copy last non zero entry to all entries after
    for i in n_cp..thermal_list.len() {
        // not necessary, just double check
        if thermal_list[i].1.cp == 0.0 {
            thermal_list[i].1.cp = thermal_list[n_cp].1.cp;
        }
    }
    for i in n_r_th..thermal_list.len() {
        // not necessary, just double check
        if thermal_list[i].1.R_th == 0.0 {
            thermal_list[i].1.R_th = thermal_list[n_cp].1.R_th;
        }
    }
    for i in n_cp..thermal_list.len() {
        // not necessary, just double check
        if thermal_list[i].1.e == 0.0 {
            thermal_list[i].1.e = thermal_list[n_cp].1.e;
        }
    }
    

}

/// Adjust read values to thickness & density
fn adjust_to_geometry (layer: &mut Layer ) {
    layer.areal_density = layer.density * layer.tickness;
    for prop_list in layer.thermal_prop_layer_temp.iter_mut() {
        prop_list.1.cp *= layer.areal_density;
        prop_list.1.R_th = layer.tickness / prop_list.1.R_th * 1000.0;
    }
}

/// Copy layer into an expanded list part temperature of predefined range & steps as basis
fn create_corresponding_part_temp_list(structure: &mut Structure) {
    let temp_range = structure.temp - TEMPERATURE_EQUALIZED;

    // Calulate corresponding structure temperature for each layer
    for layer in structure.layers.iter_mut() {
        for data_pair in layer.thermal_prop_layer_temp.iter() {
            let mut temp_struct = temp_range / (layer.temp_max - TEMPERATURE_EQUALIZED) * (data_pair.0  - TEMPERATURE_EQUALIZED) + TEMPERATURE_EQUALIZED;
            if data_pair.0 > temp_struct {
                temp_struct = data_pair.0;
            }
            layer.thermal_prop_layer_in_struct.push(DataTriplet { temp_part: temp_struct, thermal_data: data_pair.1, temp_sub_part: data_pair.0 } );
        }
        let expanded_list = expand_list(&layer.thermal_prop_layer_in_struct, &structure.temp_list2);
        

        layer.thermal_prop_struct_temp.clear();
        for n in &expanded_list {
            layer.thermal_prop_struct_temp.push(n.to_data_pair())
        }
        //output_data_Triplet(&layer.name,&layer.thermal_prop_layer_in_struct, "out/test/".to_owned() + &structure.name + "/thermal_prop_layer_in_struct");
        //output_data_Triplet(&layer.name,&expanded_list, "out/test/".to_owned() + &structure.name + "/expanded_list");
    }

}

/// expand list in to predefined range & steps and fill in the gaps
fn expand_list(thermal_list: &Vec<DataTriplet>, ref_temp_list: &Vec<f32>) -> Vec<DataTriplet>{
    let mut data_adjusted: Vec<DataTriplet> = Vec::<DataTriplet>::new();

        for temp in ref_temp_list {
            if temp >= &thermal_list[0].temp_part{
                break;
            }
            data_adjusted.push(DataTriplet { temp_part: *temp, thermal_data: thermal_list[0].thermal_data, temp_sub_part: 0.0 })
        }

        let mut n = 0;
        for (i, row) in thermal_list.iter().enumerate() {
            let temp_delta = thermal_list[i+1].temp_part - row.temp_part;
            let data_delta = thermal_list[i+1].thermal_data - row.thermal_data;
            let temp_sub_part_delta = thermal_list[i+1].temp_sub_part - row.temp_sub_part;
            for j in n..ref_temp_list.len() {
                if ref_temp_list[j] == row.temp_part {
                    n = j;
                    data_adjusted.push(DataTriplet { temp_part: row.temp_part, 
                                                    thermal_data: row.thermal_data,
                                                    temp_sub_part: row.temp_sub_part })
                } 
                else if ref_temp_list[j] > row.temp_part && ref_temp_list[j] < thermal_list[i+1].temp_part {
                    n = j;
                    let data = data_delta / temp_delta * (ref_temp_list[j] - row.temp_part) + row.thermal_data;
                    let temp_sub_part = temp_sub_part_delta / temp_delta * (ref_temp_list[j] - row.temp_part) + row.temp_sub_part;
                    data_adjusted.push(DataTriplet { temp_part: ref_temp_list[j], 
                                                     thermal_data: data,
                                                     temp_sub_part: temp_sub_part })
                } 
            }
            if i+2 == thermal_list.len() {
                    let data = data_delta / temp_delta * (ref_temp_list[n+1] - row.temp_part) + row.thermal_data;
                    let temp_sub_part = temp_sub_part_delta / temp_delta * (ref_temp_list[n+1] - row.temp_part) + row.temp_sub_part;
                    data_adjusted.push(DataTriplet { temp_part: ref_temp_list[n+1], 
                                                     thermal_data: data,
                                                     temp_sub_part: temp_sub_part });
                break;
            }
        }
        for temp in ref_temp_list.iter().skip(n+2){
            data_adjusted.push(DataTriplet { temp_part: *temp, thermal_data: data_adjusted[n+1].thermal_data, temp_sub_part: 0.0 })
        }

    data_adjusted
}

/// calculate the part values based on data from its structures
fn calculate_part(part: &mut Construction) {
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

    for (i, temp) in part.structures[1].0.temp_list2.iter().enumerate() {
        let mut cp = 0.0;
        let mut r_th = 0.0;
        let mut e = 0.0;

        for (structure, portion) in part.structures.iter() {
            cp += structure.data[i].1.cp * structure.areal_density / part.areal_density * portion * (structure.temp-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
            r_th += structure.data[i].1.R_th * portion * (structure.temp-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
            e += structure.data[i].1.e * portion * (structure.temp-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
        }
        part.data.push(DataPair((*temp - 25.0) as f32, Data{cp, R_th: 1.0 / r_th, e: e}));
    }
}

/// calculate the structure values based on data from layer
fn calculate_structure(structure: &mut Structure) {
    structure.areal_density = 0.0;
    structure.tickness = 0.0;
    
    for layer in structure.layers.iter() {
        structure.areal_density += layer.areal_density; 
        structure.tickness += layer.tickness;
    }
    adjust_layers_to_structure_temp_and_density(structure);

    for (i, temp) in structure.temp_list2.iter().enumerate() {
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
        structure.data.push(DataPair(*temp, Data{cp, R_th: r_th, e: e}));
    }
}


/// multiplyer on layer values based om structure temperature & density
fn adjust_layers_to_structure_temp_and_density (structure: &mut Structure) {
    for layer in structure.layers.iter_mut() {
        layer.thermal_prop_struct_temp_frac = layer.thermal_prop_struct_temp.clone();

        for (i, prop_list) in layer.thermal_prop_struct_temp_frac.iter_mut().enumerate() {
            prop_list.1.cp *= prop_list.0 / (structure.temp_list2[i] * structure.areal_density) ;
            prop_list.1.R_th *= prop_list.0 / structure.temp_list2[i];
        }
    }
}