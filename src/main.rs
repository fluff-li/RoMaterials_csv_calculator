mod read_write;
mod data_holder;

use read_write::*;
use data_holder::*;

use std::{
    error::Error,
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
    let tps_paths = get_files("bib/tps".to_string(), OsString::from("csv"));

    for path in tps_paths.iter() {
        let mut tps = read_tps_csv(path);

        for segment in tps.segments_min.iter_mut() {
            read_material_csv(segment).unwrap();
            fill_gaps_in_csv(&mut segment.data_csv);
            segment.areal_density = (segment.density * segment.tickness + segment.additive_areal_weight) * segment.portion ;
            
            //println!("{}, {}", tps.name, segment.name);
            segment.data_tps_temp_map = map_component_data_to_assembly(tps.temp, segment.temp_hot_side, &segment.data_csv, &temp_list);
            segment.data_height_adjust = adjust_to_height(segment.tickness * segment.portion, &segment.data_tps_temp_map);
            segment.data_avg_r = avg_cp_k(segment.tickness, &segment.data_height_adjust, segment.temp_hot_side, segment.temp_cold_side);
        }
        for segment in tps.segments_max.iter_mut() {
            read_material_csv(segment).unwrap();
            fill_gaps_in_csv(&mut segment.data_csv);
            segment.areal_density = (segment.density * segment.tickness + segment.additive_areal_weight) * segment.portion ;
            
            //println!("{}, {}", tps.name, segment.name);
            segment.data_tps_temp_map = map_component_data_to_assembly(tps.temp, segment.temp_hot_side, &segment.data_csv, &temp_list);
            segment.data_height_adjust = adjust_to_height(segment.tickness * segment.portion, &segment.data_tps_temp_map);
            segment.data_avg_r = avg_cp_k(segment.tickness, &segment.data_height_adjust, segment.temp_hot_side, segment.temp_cold_side);
        }
        calc_tps_height_density(&mut tps);
        for segment in tps.segments_min.iter_mut() {
            segment.data_tps_temp_mult = tps_value_mult(tps.areal_density_min,segment.areal_density ,&segment.data_avg_r);
        }
        for segment in tps.segments_max.iter_mut() {
            segment.data_tps_temp_mult = tps_value_mult(tps.areal_density_max,segment.areal_density ,&segment.data_avg_r);
        }
        tps.data_min = calc_tps_data(&tps.segments_min, &temp_list);
        tps.data_max = calc_tps_data(&tps.segments_max, &temp_list);
        

        output_tps(&tps, OUTPUT_DIRECTORY.to_string()).unwrap();
        tps_list.push(tps);
    }
    

    let part_paths = get_files("bib/part".to_string(), OsString::from("csv"));
    for path in part_paths.iter() {
        let (mut part, mut part_list_min) = read_part_csv(path);

        for tps in &tps_list {
            for (i, (name, portion, height_min, height_max)) in part_list_min.iter_mut().enumerate() {
                if tps.name == *name {
                    let tps_new = tps_change_height(&tps,*height_min, *height_max );
                    let data_min = map_component_data_to_assembly(part.temp, tps.temp, &tps_new.data_min, &temp_list);
                    let data_max = map_component_data_to_assembly(part.temp, tps.temp, &tps_new.data_max, &temp_list);

                    part.tps_list.push((tps_new, *portion, data_min, data_max));
                    part_list_min.remove(i);
                    break;
                }
            }

            if part_list_min.is_empty() {
                break;
            }
        }
        if !part_list_min.is_empty() {
            println!("Error Part {}: Structures not found. {:?}", part.name, part_list_min);
            process::exit(1);
        }

    
        calculate_part(&mut part, &temp_list);

        output_part(part, OUTPUT_DIRECTORY.to_string()).unwrap();
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
            thermal_list[i].1.R_th = thermal_list[n_r_th].1.R_th;
        }
    }
    for i in n_e..thermal_list.len() {
        // not necessary, just double check
        if thermal_list[i].1.e == 0.0 {
            thermal_list[i].1.e = thermal_list[n_e].1.e;
        }
    }
}

/// Adjust read values to thickness & density
fn adjust_to_height ( height: f32, data: &Vec<DataTriplet> ) -> Vec<DataTriplet> {
    let mut data_new = Vec::<DataTriplet>::new();
    for row in data.iter() {
        let mut data = row.thermal_data.clone();
        if row.thermal_data.R_th != 0.0 {
            data.R_th = height / row.thermal_data.R_th * 1000.0;
        } else {

        }
        data_new.push(DataTriplet{temp_part: row.temp_part, temp_sub_part: row.temp_sub_part, thermal_data: data});
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

    // fill copy of last usable entry to fill the rest
    for temp in ref_temp_list.iter().skip(index){
        let temp_sub_part = data_adjusted.last().unwrap().temp_sub_part / data_adjusted.last().unwrap().temp_part * temp;
        data_adjusted.push(DataTriplet { temp_part: *temp, thermal_data: data_adjusted.last().unwrap().thermal_data, temp_sub_part: temp_sub_part})
    }

    data_adjusted
}

/// calculate the part values based on data from its structures
fn calculate_part(part: &mut Part, temp_ref_list: &Vec<f32>) {
    part.areal_density_min = 0.0;
    part.areal_density_max = 0.0;
    part.height_min = 0.0;
    part.height_max = 0.0;

    for (tps, portion, _data_min, _data_max) in part.tps_list.iter() {
        part.areal_density_min += tps.areal_density_min * portion;
        part.areal_density_max += tps.areal_density_max * portion;

        part.height_min += tps.tickness_min * portion;
        part.height_max += tps.tickness_max * portion;
    }

    for (i, temp) in temp_ref_list.iter().enumerate() {
        let mut cp_min = 0.0;
        let mut r_th_min = 0.0;
        let mut e_min = 0.0;
        let mut cp_max = 0.0;
        let mut r_th_max = 0.0;
        let mut e_max = 0.0;

        for (tps, portion, data_min, data_max) in part.tps_list.iter() {
            cp_min += data_min[i].thermal_data.cp * tps.areal_density_min / part.areal_density_min * portion * (data_min[i].temp_sub_part - TEMPERATURE_EQUALIZED) / (data_min[i].temp_part - TEMPERATURE_EQUALIZED);
            r_th_min += portion * data_min[i].thermal_data.R_th;
            e_min += data_min[i].thermal_data.e * portion;

            cp_max += data_max[i].thermal_data.cp * tps.areal_density_max / part.areal_density_max * portion * (data_max[i].temp_sub_part - TEMPERATURE_EQUALIZED) / (data_max[i].temp_part - TEMPERATURE_EQUALIZED);
            r_th_max += portion * data_max[i].thermal_data.R_th;
            e_max += data_max[i].thermal_data.e * portion;
        }
        part.data_min.push(DataPair((*temp - 25.0) as f32, Data{cp: cp_min, R_th: 1.0 / r_th_min, e: e_min}));
        part.data_max.push(DataPair((*temp - 25.0) as f32, Data{cp: cp_max, R_th: 1.0 / r_th_max, e: e_max}));
    } 
}

/// calculate the structure values based on data from layer
fn calc_tps_height_density(tps: &mut TPS) {
    tps.areal_density_min = 0.0;
    tps.tickness_min = 0.0;
    tps.areal_density_max = 0.0;
    tps.tickness_max = 0.0;
    
    for layer in tps.segments_min.iter() {
        tps.areal_density_min += layer.areal_density; 
        tps.tickness_min += layer.tickness;
    }

    for layer in tps.segments_max.iter() {
        tps.areal_density_max += layer.areal_density; 
        tps.tickness_max += layer.tickness;
    }
}

fn calc_tps_data(segments: &Vec<Segment>, temp_list: &Vec<f32>) -> Vec<DataPair> {

    let mut data = Vec::<DataPair>::new();

    for (i, temp) in temp_list.iter().enumerate() {    
        let mut cp = 0.0;
        let mut r_th = 0.0;
        let mut e = 0.0;

        for layer in segments.iter() {
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
    let density_frac = segment_density / assembly_density;

    for row in segment_data.iter() {
        new_tripl.push(DataTriplet{temp_part: row.temp_part, temp_sub_part: row.temp_sub_part, 
                        thermal_data: Data{cp: row.thermal_data.cp * row.temp_sub_part * density_frac / row.temp_part,
                                            R_th: row.thermal_data.R_th, //prop_list.thermal_data.R_th *= prop_list.temp_sub_part / prop_list.temp_part;
                                            e: row.thermal_data.e,
        }})
    }
    new_tripl
}

fn map_component_data_to_assembly(assemb_temp_max: f32, comp_temp_max: f32, comp_data: &Vec<DataPair>, temp_list: &Vec<f32>) -> Vec<DataTriplet> {
    let mut data_new = Vec::<DataTriplet>::new();
    let temp_mult;
    if comp_temp_max < assemb_temp_max {
        temp_mult = (assemb_temp_max - TEMPERATURE_EQUALIZED) / (comp_temp_max - TEMPERATURE_EQUALIZED);
    } else {
        temp_mult = 1.0;
    }
    
    for data in comp_data.iter() {
        let temp_assemb = (data.0 - TEMPERATURE_EQUALIZED) * temp_mult + TEMPERATURE_EQUALIZED;
        data_new.push(DataTriplet{temp_part: temp_assemb, thermal_data: data.1, temp_sub_part: data.0})
    }
    //data_new
    fit_list(&data_new, &temp_list)
}

/// Returns a new list with an average conductivity accross segment for given cold & Hot Side Temperature 
pub fn avg_cp_k(lenght: f32, data: &Vec<DataTriplet>, temp_max: f32, temp_min: f32 ) -> Vec<DataTriplet>{
    let mut data_out= data.clone();
    let mut steps = Vec::<(f32,f32,f32,f32,f32)>::new();

    let temp_frac = temp_min / temp_max;

    // set a refrence k for q = 1;
    let mut k_ref = 0.0;
    let mut t_ref = 0.0;
    // q = T * d/ k; -> d = q * k / T
    for (i, _row) in data.iter().enumerate() {
        if temp_max <= data[i].temp_sub_part + 25.0 && temp_max >= data[i].temp_sub_part -25.0 {
            t_ref = data[i].temp_sub_part ;
            if data[i].thermal_data.R_th == 0.0 {
                return data.clone();
            }
            k_ref = lenght / data[i].thermal_data.R_th  * 1000.0;
            //println!("k_ref {}, t_ref {} ",k_ref ,t_ref);
            break;
        }
    }
    

    // extrapolate d value for the rest
    let q_ref  = t_ref * 1.0 / k_ref; // 1.0 = d_ref
    let mut d_sum: f32 = 0.0;
    for row in data.iter() {
        let k = lenght / row.thermal_data.R_th * 1000.0;
        let d = q_ref * k / row.temp_sub_part ;
        d_sum += d;
        //println!("k_ref {}, t_ref {}",k ,row.temp_sub_part);
        steps.push((row.temp_sub_part ,k , d, d_sum, row.thermal_data.cp));
    }
    
    //let mut steps2 = Vec::<(f32,f32,f32)>::new();
    for (n, row) in data_out.iter_mut().enumerate() {
        let mut i = 0;
        let mut r_th = 0.0;
        let mut cp = 0.0;
        let mut d_sum = 0.0;
        while i < steps.len(){
            if steps[i].0 >= steps[n].0 * temp_frac && steps[i].0 <= steps[n].0 {
                r_th += steps[i].2 / steps[i].1 * 1000.0;
                cp += steps[i].4 * steps[i].2;
                d_sum += steps[i].2
            }
            i += 1;
        }
        //steps2.push((steps[n].0, r_th / d_sum * lenght , steps[n].0 * temp_frac));
        row.thermal_data.R_th = r_th / d_sum * lenght;
        row.thermal_data.cp = cp / d_sum;
    }
    data_out
}

fn tps_change_height(tps_ref: &TPS, new_height_min: f32, new_height_max: f32) -> TPS {
    let mut tps = tps_ref.clone();

    if new_height_min == f32::INFINITY {
        tps.tickness_min = tps.tickness_max;
        tps.areal_density_min = tps.areal_density_max;

        for (i, data) in tps.data_min.iter_mut().enumerate() {
            data.1.cp = tps.data_max[i].1.cp;
            data.1.R_th = tps.data_max[i].1.R_th;
        }
    } else if new_height_min != f32::NEG_INFINITY {
        let height_factor = (new_height_min - tps_ref.tickness_min) / (tps_ref.tickness_max - tps_ref.tickness_min);

        if height_factor > 0.001 && height_factor < 0.999 {
            tps.tickness_min = (tps.tickness_max - tps.tickness_min) * height_factor + tps.tickness_min;
            tps.areal_density_min = (tps.areal_density_max - tps.areal_density_min) * height_factor + tps.areal_density_min;

            for (i, data) in tps.data_min.iter_mut().enumerate() {
                data.1.cp = (tps.data_max[i].1.cp - data.1.cp) * height_factor + data.1.cp;
                data.1.R_th = (tps.data_max[i].1.R_th - data.1.R_th) * height_factor + data.1.R_th;
            }
        }
    }
    if new_height_max == f32::NEG_INFINITY {
        tps.tickness_max = tps.tickness_min;
        tps.areal_density_max = tps.areal_density_min;

        for (i, data) in tps.data_max.iter_mut().enumerate() {
            data.1.cp = tps.data_min[i].1.cp;
            data.1.R_th = tps.data_min[i].1.R_th;
        }
    } else if new_height_max != f32::INFINITY {
        let height_factor = (new_height_max - tps_ref.tickness_min) / (tps_ref.tickness_max - tps_ref.tickness_min);

        if height_factor < 0.999 && height_factor > 0.001 {
            tps.tickness_max = (tps.tickness_max - tps.tickness_min) * height_factor + tps.tickness_min;
            tps.areal_density_max = (tps.areal_density_max - tps.areal_density_min) * height_factor + tps.areal_density_min;

            for (i, data) in tps.data_max.iter_mut().enumerate() {
                data.1.cp = (data.1.cp - tps.data_min[i].1.cp) * height_factor + tps.data_min[i].1.cp;
                data.1.R_th = (data.1.R_th - tps.data_min[i].1.R_th) * height_factor + tps.data_min[i].1.R_th;
            }
        }
    }
    tps
}