mod read_write;
mod data_holder;

use read_write::*;
use data_holder::*;

use std::{
    process,
    path::PathBuf, 
    ffi::OsString,
};


const TEMPERATURE_EQUALIZED: f32 = 0.0;//273.15;
const TEMP_STEP: f32 = 50.0;
const TEMP_LIST: &str = "bib/Temp_List.csv";
const OUTPUT_DIRECTORY: &str = "out/";


fn main() -> std::io::Result<()> {
    let mut tps_list = Vec::<TPS>::new();
    let temp_list = read_temp_list_csv2(&PathBuf::from(TEMP_LIST));
    let tps_paths = get_files("bib/structures".to_string(), OsString::from("csv"));

    for path in tps_paths.iter() {
        let mut tps = read_structure_csv(path);
        tps.temp_list2 = temp_list.clone();

        for segment in tps.segments.iter_mut() {
            read_material_csv(segment);
            fill_gaps_in_csv(&mut segment.data_csv);
            segment.areal_density = segment.density * segment.tickness;
            segment.data_height_adjust = adjust_to_height(segment.areal_density, segment.tickness, &segment.data_csv);
            segment.data_tps_temp_map = map_component_data_to_assembly(tps.temp_max, segment.temp_max, &segment.data_height_adjust, &temp_list);            
        }
        calc_tps_height_density(&mut tps);
        for segment in tps.segments.iter_mut() {
            segment.data_tps_temp_mult = tps_value_mult(tps.areal_density,segment.areal_density ,&segment.data_tps_temp_map);
        }
        tps.data = calc_tps_data(&tps, &temp_list);

        output_tps(&tps, OUTPUT_DIRECTORY.to_string() + "structures/").unwrap();
        tps_list.push(tps);
    }


    let part_paths = get_files("bib/part".to_string(), OsString::from("csv"));
    for path in part_paths.iter() {
        let (mut part, mut part_list_min, mut part_list_max) = read_part_csv(path);

        for structure in &tps_list {
            for (i, (name, portion)) in part_list_min.iter_mut().enumerate() {
                if structure.name == *name {
                    part.tps_list_min.push((structure.clone(), *portion
                                            , map_component_data_to_assembly(part.temp, structure.temp_max, &structure.data, &temp_list)));
                    part_list_min.remove(i);
                    output_data_Triplet(&structure.name,&part.tps_list_min.last().unwrap().2, "out/test/".to_owned() + &part.name + "/expanded_list2");
                    break;
                }
            }
            for (i, (name, portion)) in part_list_max.iter_mut().enumerate() {
                if structure.name == *name {
                    part.tps_list_max.push((structure.clone(), *portion
                                            , map_component_data_to_assembly(part.temp, structure.temp_max, &structure.data, &temp_list)));
                    part_list_max.remove(i);
                    break;
                }
            }
            if part_list_min.is_empty() & part_list_max.is_empty() {
                break;
            }
        }
        if !part_list_min.is_empty() | !part_list_max.is_empty()  {
            println!("Error Part {}: Structures not found. {:?} {:?}", part.name, part_list_min, part_list_max );
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
fn adjust_to_height (areal_density: f32, height: f32, data: &Vec<DataPair> ) -> Vec<DataPair> {
    let mut data_new = Vec::<DataPair>::new();
    for row in data.iter() {
        let r_th;
        if row.1.R_th != 0.0 {
            r_th = height / row.1.R_th * 1000.0;
        } else {
            r_th = 0.0;
        }
        data_new.push(DataPair(row.0, Data{cp: row.1.cp, R_th: r_th, e: row.1.e }));
    }
    data_new
}

/// expand list in to predefined range & steps and fill in the gaps
fn fit_list(thermal_list: &Vec<DataTriplet>, ref_temp_list: &Vec<f32>) -> Vec<DataTriplet>{
    //let mut data_adjusted: Vec<DataTriplet> = Vec::<DataTriplet>::with_capacity(ref_temp_list.len());
    let mut data_adjusted = Vec::<DataTriplet>::new();
    let mut index = 0;
    let mut index2 = 0;

    for (i, temp) in ref_temp_list.iter().enumerate() {
        if temp >= &thermal_list[0].temp_part{
            index = i;
            break;
        }
        data_adjusted.push( DataTriplet{ temp_part: *temp, thermal_data: thermal_list[0].thermal_data, temp_sub_part: thermal_list[0].temp_sub_part * *temp / thermal_list[0].temp_part });
    }

    // take two neighboring values and fill int for temperatures fitting in between
    for (n, row) in thermal_list.iter().enumerate() {
        if row.temp_part > *ref_temp_list.last().unwrap() {
            index2 = n;
            break;
        }
        
        let temp_delta = thermal_list[n+1].temp_part - row.temp_part;
        let data_delta = thermal_list[n+1].thermal_data - row.thermal_data;
        let temp_sub_part_delta = thermal_list[n+1].temp_sub_part - row.temp_sub_part;


        for (i, temp) in ref_temp_list.iter().skip(index).enumerate() {
            if *temp == row.temp_part {
                data_adjusted.push(DataTriplet { temp_part: row.temp_part, 
                                                thermal_data: row.thermal_data,
                                                temp_sub_part: row.temp_sub_part });
                index += 1; 
            }
            if *temp > row.temp_part && *temp < thermal_list[n + 1].temp_part {
                let data = data_delta / temp_delta * (temp - row.temp_part) + row.thermal_data;
                    let temp_sub_part = temp_sub_part_delta / temp_delta * (temp - row.temp_part) + row.temp_sub_part;
                    data_adjusted.push(DataTriplet { temp_part: *temp, 
                                                     thermal_data: data,
                                                     temp_sub_part: temp_sub_part });
                index += 1;
            }
        }
        if index == thermal_list.len() {
            break;
        }
        if n + 2 == thermal_list.len() {
            let data = data_delta / temp_delta * (ref_temp_list[index] - row.temp_part) + row.thermal_data;
                    let temp_sub_part = temp_sub_part_delta / temp_delta * (ref_temp_list[index] - row.temp_part) + row.temp_sub_part;
                    data_adjusted.push( DataTriplet { temp_part: ref_temp_list[index], 
                                                     thermal_data: data,
                                                     temp_sub_part: temp_sub_part });
            index += 1;
            index2 = n;
            break;
        }

    }

    // fill copy of last usable to fill the rest
    for temp in ref_temp_list.iter().skip(index){
        let temp_sub_part = data_adjusted.last().unwrap().temp_sub_part / data_adjusted.last().unwrap().temp_part * temp;
        data_adjusted.push(DataTriplet { temp_part: *temp, thermal_data: data_adjusted.last().unwrap().thermal_data, temp_sub_part: temp_sub_part})
    }

    data_adjusted
}

/// calculate the part values based on data from its structures
fn calculate_part(part: &mut Part) {
    part.areal_density_min = 0.0;
    part.areal_density_max = 0.0;
    part.height_min0 = f32::INFINITY;
    part.height_min1 = 0.0;
    part.height_max0 = f32::INFINITY;
    part.height_max1 = 0.0;

    for (structure, portion, data) in part.tps_list_min.iter() {
        part.areal_density_min += structure.areal_density * portion;

        if part.height_min0 > structure.tickness {
            part.height_min0 = structure.tickness;
        }
        if part.height_min1 < structure.tickness {
            part.height_min1 = structure.tickness;
        }   
    }
    for (structure, portion, data) in part.tps_list_max.iter() {
        part.areal_density_max += structure.areal_density * portion;

        if part.height_max0 > structure.tickness {
            part.height_max0 = structure.tickness;
        }
        if part.height_max1 < structure.tickness {
            part.height_max1 = structure.tickness;
        }   
    }

    for (i, temp) in part.tps_list_min[1].0.temp_list2.iter().enumerate() {
        let mut cp = 0.0;
        let mut r_th = 0.0;
        let mut e = 0.0;

        for (structure, portion, data_adjusted) in part.tps_list_min.iter() {
            cp += data_adjusted[i].thermal_data.cp * structure.areal_density / part.areal_density_min * portion * (structure.temp_max - TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
            r_th += data_adjusted[i].thermal_data.R_th * portion;// * (structure.temp_max - TEMPERATURE_EQUALIZED) / (part.temp - TEMPERATURE_EQUALIZED);
            e += data_adjusted[i].thermal_data.e * portion; // * (structure.temp_max-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
            //e +=  data_adjusted[i].thermal_data.e * portion * ( f32::powf(structure.temp_max,4.0) - f32::powf(0.0, 4.0) ) / ( f32::powf(part.temp,4.0) - f32::powf(0.0, 4.0) );
        }
        part.data_min.push(DataPair((*temp - 25.0) as f32, Data{cp: cp, R_th: 1.0 / r_th, e: e}));
    }
    for (i, temp) in part.tps_list_max[1].0.temp_list2.iter().enumerate() {
        let mut cp = 0.0;
        let mut r_th = 0.0;
        let mut e = 0.0;

        for (structure, portion, data_adjusted) in part.tps_list_max.iter() {
            cp += data_adjusted[i].thermal_data.cp * structure.areal_density / part.areal_density_max * portion * (structure.temp_max-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
            r_th += data_adjusted[i].thermal_data.R_th * portion;// * (structure.temp_max - TEMPERATURE_EQUALIZED) / (part.temp - TEMPERATURE_EQUALIZED);
            e += data_adjusted[i].thermal_data.e * portion; // * (structure.temp_max-TEMPERATURE_EQUALIZED) / (part.temp-TEMPERATURE_EQUALIZED);
        }
        part.data_max.push(DataPair((*temp - 25.0) as f32, Data{cp: cp, R_th: 1.0 / r_th, e: e}));
    }
}

/// calculate the structure values based on data from layer
fn calc_tps_height_density(tps: &mut TPS) {
    tps.areal_density = 0.0;
    tps.tickness = 0.0;
    
    for layer in tps.segments.iter() {
        tps.areal_density += layer.areal_density; 
        tps.tickness += layer.tickness;
    }
}

fn calc_tps_data(tps: &TPS, temp_list: &Vec<f32>) -> Vec<DataPair> {
    let mut data = Vec::<DataPair>::new();

    for (i, temp) in temp_list.iter().enumerate() {    
        let mut cp = 0.0;
        let mut r_th = 0.0;
        let mut e = 0.0;

        for layer in tps.segments.iter() {
            cp += layer.data_tps_temp_mult[i].thermal_data.cp;
            r_th += layer.data_tps_temp_mult[i].thermal_data.R_th;

            if e <= 0.0 {
                e = layer.data_tps_temp_mult[i].thermal_data.e;
            }
        }
        data.push(DataPair(*temp, Data{cp, R_th: r_th, e: e}));
    }
    data
}


/// multiplyer on component values based om assembly temperature & density
fn tps_value_mult (assembly_density: f32, segment_density: f32, segment_data: &Vec<DataTriplet>) -> Vec<DataTriplet>{
    let mut new_tripl = Vec::<DataTriplet>::new();

    for row in segment_data.iter() {
        
        new_tripl.push(DataTriplet{temp_part: row.temp_part, temp_sub_part: row.temp_sub_part, 
                        thermal_data: Data{cp: row.thermal_data.cp * row.temp_sub_part * segment_density / (row.temp_part * assembly_density),
                                            R_th: row.thermal_data.R_th, //prop_list.thermal_data.R_th *= prop_list.temp_sub_part / prop_list.temp_part;
                                            e: row.thermal_data.e,
        }})
    }
    new_tripl
}

fn map_component_data_to_assembly(assemb_temp_max: f32, comp_temp_max: f32, comp_data: &Vec<DataPair>, temp_list: &Vec<f32>) -> Vec<DataTriplet> {
    let temp_range = assemb_temp_max - TEMPERATURE_EQUALIZED;
    let mut data_new = Vec::<DataTriplet>::new();

    for data in comp_data.iter() {
        let temp_assemb = (data.0 - TEMPERATURE_EQUALIZED) * (assemb_temp_max - TEMPERATURE_EQUALIZED) / (comp_temp_max - TEMPERATURE_EQUALIZED) + TEMPERATURE_EQUALIZED;
        data_new.push(DataTriplet{temp_part: temp_assemb, thermal_data: data.1, temp_sub_part: data.0})
    }
    //data_new
    fit_list(&data_new, &temp_list)
}